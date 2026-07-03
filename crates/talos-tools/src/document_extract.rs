//! Document extraction tool for local files.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use talos_core::tool::{
    AgentTool, ToolFamily, ToolNature, ToolPermissionFacet, ToolResourceKind, ToolResult,
};
use talos_core::tool_parameters;
use thiserror::Error;

use super::FileToolError;
use crate::file_tools::resolve_workspace_path;

const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
const DEFAULT_MAX_BYTES: usize = 32768;
const MIN_MAX_BYTES: usize = 1;
const MAX_MAX_BYTES: usize = 65536;
const TRUNCATION_MARKER: &str = "\n[... truncated: {total} bytes total, showing {shown} ...]";

#[derive(Debug, Error)]
pub enum DocumentExtractError {
    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("path escape: {0}")]
    PathEscape(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

fn default_max_bytes() -> Option<usize> {
    Some(DEFAULT_MAX_BYTES)
}

fn default_format() -> Option<String> {
    Some("auto".to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentExtractInput {
    pub path: String,

    #[serde(default = "default_max_bytes")]
    #[schemars(range(min = 1, max = 65536))]
    pub max_bytes: Option<usize>,

    #[serde(default = "default_format")]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetectedFormat {
    Text,
    Markdown,
    Html,
    Json,
    JsonLines,
    Csv,
    Xml,
    Pdf,
    Office,
    Image,
    Archive,
    Binary,
    Unknown,
}

impl DetectedFormat {
    fn as_str(&self) -> &'static str {
        match self {
            DetectedFormat::Text => "text",
            DetectedFormat::Markdown => "markdown",
            DetectedFormat::Html => "html",
            DetectedFormat::Json => "json",
            DetectedFormat::JsonLines => "jsonl",
            DetectedFormat::Csv => "csv",
            DetectedFormat::Xml => "xml",
            DetectedFormat::Pdf => "pdf",
            DetectedFormat::Office => "office",
            DetectedFormat::Image => "image",
            DetectedFormat::Archive => "archive",
            DetectedFormat::Binary => "binary",
            DetectedFormat::Unknown => "unknown",
        }
    }
}

fn extension_to_format(ext: &str) -> Option<DetectedFormat> {
    match ext.to_lowercase().as_str() {
        "txt" | "log" | "text" | "cfg" | "ini" | "env" | "toml" | "yaml" | "yml" | "conf" => {
            Some(DetectedFormat::Text)
        }
        "md" | "markdown" => Some(DetectedFormat::Markdown),
        "html" | "htm" | "xhtml" => Some(DetectedFormat::Html),
        "json" => Some(DetectedFormat::Json),
        "jsonl" | "ndjson" => Some(DetectedFormat::JsonLines),
        "csv" | "tsv" | "tab" => Some(DetectedFormat::Csv),
        "xml" => Some(DetectedFormat::Xml),
        "pdf" => Some(DetectedFormat::Pdf),
        "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" => {
            Some(DetectedFormat::Office)
        }
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tif" | "tiff" | "heic" => {
            Some(DetectedFormat::Image)
        }
        "zip" | "tar" | "gz" | "tgz" | "bz2" | "xz" | "7z" | "rar" => Some(DetectedFormat::Archive),
        _ => None,
    }
}

fn parse_format_hint(hint: &str) -> Option<DetectedFormat> {
    match hint.to_lowercase().as_str() {
        "text" => Some(DetectedFormat::Text),
        "markdown" | "md" => Some(DetectedFormat::Markdown),
        "html" | "htm" => Some(DetectedFormat::Html),
        "json" => Some(DetectedFormat::Json),
        "jsonl" | "ndjson" | "json-lines" => Some(DetectedFormat::JsonLines),
        "csv" | "tsv" => Some(DetectedFormat::Csv),
        "xml" => Some(DetectedFormat::Xml),
        "pdf" => Some(DetectedFormat::Pdf),
        "office" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" => Some(DetectedFormat::Office),
        "image" | "png" | "jpg" | "jpeg" | "gif" | "webp" => Some(DetectedFormat::Image),
        "archive" | "zip" => Some(DetectedFormat::Archive),
        "auto" | "" => None,
        _ => Some(DetectedFormat::Unknown),
    }
}

fn sniff_unsupported_binary(bytes: &[u8]) -> Option<DetectedFormat> {
    if bytes.starts_with(b"%PDF-") {
        return Some(DetectedFormat::Pdf);
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || bytes.starts_with(b"\xff\xd8\xff")
        || bytes.starts_with(b"GIF87a")
        || bytes.starts_with(b"GIF89a")
        || (bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP")
    {
        return Some(DetectedFormat::Image);
    }
    if bytes.starts_with(b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1") {
        return Some(DetectedFormat::Office);
    }
    if bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06") {
        return Some(DetectedFormat::Archive);
    }

    None
}

fn looks_like_html(bytes: &[u8]) -> bool {
    let s = String::from_utf8_lossy(bytes);
    let lower = s.to_lowercase();
    lower.starts_with("<!doctype") || lower.starts_with("<html") || lower.contains("<html")
}

fn looks_like_json(bytes: &[u8]) -> bool {
    let s = String::from_utf8_lossy(bytes);
    let trimmed = s.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

fn detect_format(path: &Path, hint: Option<&str>, first_bytes: &[u8]) -> DetectedFormat {
    if let Some(h) = hint
        && h != "auto"
        && let Some(fmt) = parse_format_hint(h)
    {
        return fmt;
    }

    if let Some(ext) = path.extension().and_then(|e| e.to_str())
        && let Some(fmt) = extension_to_format(ext)
    {
        return fmt;
    }

    if let Some(fmt) = sniff_unsupported_binary(first_bytes) {
        return fmt;
    }

    if first_bytes.contains(&0u8) {
        return DetectedFormat::Binary;
    }
    if looks_like_html(first_bytes) {
        return DetectedFormat::Html;
    }
    if looks_like_json(first_bytes) {
        return DetectedFormat::Json;
    }

    DetectedFormat::Text
}

fn bound_content(content: &str, max_bytes: usize) -> (String, bool) {
    let total_bytes = content.len();
    if total_bytes <= max_bytes {
        return (content.to_string(), false);
    }

    let mut shown = max_bytes;
    for (idx, _) in content.char_indices() {
        if idx > max_bytes {
            shown = idx;
            break;
        }
    }

    let truncated_content = content[..shown].to_string();
    let marker = TRUNCATION_MARKER
        .replace("{total}", &total_bytes.to_string())
        .replace("{shown}", &shown.to_string());
    (format!("{truncated_content}{marker}"), true)
}

fn extract_text(content: &str, max_bytes: usize) -> (String, bool) {
    bound_content(content, max_bytes)
}

fn extract_markdown(content: &str, max_bytes: usize) -> (String, bool) {
    let body = strip_yaml_frontmatter(content);
    bound_content(body, max_bytes)
}

fn strip_yaml_frontmatter(content: &str) -> &str {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---\n") && !trimmed.starts_with("---\r\n") {
        return content;
    }

    let rest = if let Some(s) = trimmed.strip_prefix("---\r\n") {
        s
    } else {
        trimmed.strip_prefix("---\n").unwrap_or(trimmed)
    };

    for line in rest.lines() {
        if line.trim() == "---" {
            let pos = rest.find(line).unwrap_or(0) + line.len();
            let after = &rest[pos..];
            return after
                .strip_prefix('\n')
                .or_else(|| after.strip_prefix("\r\n"))
                .unwrap_or(after);
        }
    }

    content
}

fn extract_html_text(content: &str, max_bytes: usize) -> (String, bool) {
    let document = scraper::Html::parse_document(content);

    let body_selector = scraper::Selector::parse("body").expect("'body' is a valid CSS selector");
    let root = document
        .root_element()
        .select(&body_selector)
        .next()
        .unwrap_or_else(|| document.root_element());

    let text_items: Vec<String> = root
        .text()
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();

    if text_items.is_empty() {
        return (
            "[No visible text content extracted from HTML]".to_string(),
            false,
        );
    }

    let mut result = String::new();
    for item in &text_items {
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(item);
    }

    bound_content(&result, max_bytes)
}

fn extract_json(content: &str, max_bytes: usize) -> (String, bool) {
    match serde_json::from_str::<Value>(content) {
        Ok(val) => {
            let pretty = serde_json::to_string_pretty(&val).unwrap_or_else(|_| content.to_string());
            bound_content(&pretty, max_bytes)
        }
        Err(_) => {
            let warning = "[Warning: JSON parse failed, returning raw content]\n";
            let combined = format!("{warning}{content}");
            bound_content(&combined, max_bytes)
        }
    }
}

fn extract_jsonl(content: &str, max_bytes: usize) -> (String, bool) {
    let lines: Vec<&str> = content.lines().collect();
    let total_lines = lines.len();

    let mut result = format!("JSON Lines file: {total_lines} lines\n\n");

    for (i, line) in lines.iter().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let preview = match serde_json::from_str::<Value>(line) {
            Ok(val) => serde_json::to_string_pretty(&val).unwrap_or_else(|_| line.to_string()),
            Err(_) => line.to_string(),
        };
        result.push_str(&format!("--- Line {} ---\n{}\n", i + 1, preview));

        if result.len() > max_bytes {
            break;
        }
    }

    bound_content(&result, max_bytes)
}

fn extract_csv(content: &str, max_bytes: usize) -> (String, bool) {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return ("[Empty CSV file]".to_string(), false);
    }

    let delimiter = if lines[0].contains('\t') { '\t' } else { ',' };

    let header: Vec<&str> = lines[0].split(delimiter).collect();
    let col_count = header.len();
    let data_lines = lines.len().saturating_sub(1);

    let mut result = format!("CSV file: {col_count} columns, {data_lines} data rows\n\n");

    result.push_str(&format!("Headers: {}\n\n", header.join(" | ")));

    let max_rows = 20;
    for (i, line) in lines.iter().enumerate().skip(1).take(max_rows) {
        let cells: Vec<&str> = line.split(delimiter).collect();
        result.push_str(&format!("Row {}: {}\n", i, cells.join(" | ")));
    }

    if data_lines > max_rows {
        result.push_str(&format!("\n... and {} more rows\n", data_lines - max_rows));
    }

    bound_content(&result, max_bytes)
}

fn extract_xml(content: &str, max_bytes: usize) -> (String, bool) {
    let mut result = String::new();
    let mut in_tag = false;
    let mut last_was_tag = false;

    for ch in content.chars() {
        match ch {
            '<' => {
                in_tag = true;
                last_was_tag = true;
            }
            '>' => {
                in_tag = false;
            }
            _ if !in_tag => {
                if last_was_tag
                    && !result.is_empty()
                    && !result.ends_with(|c: char| c.is_whitespace())
                {
                    result.push(' ');
                }
                result.push(ch);
                last_was_tag = false;
            }
            _ => {}
        }
    }

    let mut collapsed = String::new();
    let mut prev_space = false;
    for ch in result.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                collapsed.push(' ');
                prev_space = true;
            }
        } else {
            collapsed.push(ch);
            prev_space = false;
        }
    }

    let text = collapsed.trim();
    if text.is_empty() {
        return ("[No text content extracted from XML]".to_string(), false);
    }

    bound_content(text, max_bytes)
}

pub struct DocumentExtractTool {
    workspace_root: PathBuf,
}

impl DocumentExtractTool {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    async fn execute_inner(&self, input: Value) -> Result<String, DocumentExtractError> {
        let extract_input: DocumentExtractInput = serde_json::from_value(input)
            .map_err(|e| DocumentExtractError::InvalidInput(e.to_string()))?;

        let path = resolve_workspace_path(&self.workspace_root, &extract_input.path).map_err(
            |e| match e {
                FileToolError::PathEscape(msg) => DocumentExtractError::PathEscape(msg),
                FileToolError::Io(e) => DocumentExtractError::Io(e),
                _ => DocumentExtractError::FileNotFound(extract_input.path.clone()),
            },
        )?;

        if !path.exists() {
            return Err(DocumentExtractError::FileNotFound(
                extract_input.path.clone(),
            ));
        }

        let metadata = std::fs::metadata(&path).map_err(DocumentExtractError::Io)?;
        let file_size = metadata.len() as usize;

        if file_size > MAX_FILE_SIZE {
            return Ok(format_metadata_only(
                &extract_input.path,
                file_size,
                DetectedFormat::Unknown,
                Some("File exceeds 10 MB extraction limit"),
            ));
        }

        let bytes = std::fs::read(&path).map_err(DocumentExtractError::Io)?;

        let sniff_bytes = &bytes[..bytes.len().min(512)];
        let format = detect_format(&path, extract_input.format.as_deref(), sniff_bytes);

        let max_bytes = extract_input
            .max_bytes
            .unwrap_or(DEFAULT_MAX_BYTES)
            .clamp(MIN_MAX_BYTES, MAX_MAX_BYTES);

        let content_str = String::from_utf8_lossy(&bytes);
        let (extracted, truncated) = match format {
            DetectedFormat::Text => extract_text(&content_str, max_bytes),
            DetectedFormat::Markdown => extract_markdown(&content_str, max_bytes),
            DetectedFormat::Html => extract_html_text(&content_str, max_bytes),
            DetectedFormat::Json => extract_json(&content_str, max_bytes),
            DetectedFormat::JsonLines => extract_jsonl(&content_str, max_bytes),
            DetectedFormat::Csv => extract_csv(&content_str, max_bytes),
            DetectedFormat::Xml => extract_xml(&content_str, max_bytes),
            DetectedFormat::Pdf
            | DetectedFormat::Office
            | DetectedFormat::Image
            | DetectedFormat::Archive
            | DetectedFormat::Binary
            | DetectedFormat::Unknown => {
                return Ok(format_metadata_only(
                    &extract_input.path,
                    file_size,
                    format,
                    Some("Unsupported format for text extraction"),
                ));
            }
        };

        let encoding = if String::from_utf8(bytes).is_ok() {
            "utf-8"
        } else {
            "utf-8 (with invalid sequences)"
        };

        let mut output = String::new();
        output.push_str(&format!("File: {}\n", extract_input.path));
        output.push_str(&format!("Size: {file_size} bytes\n"));
        output.push_str(&format!("Format: {}\n", format.as_str()));
        output.push_str(&format!("Encoding: {encoding}\n"));
        if truncated {
            output.push_str("Truncated: true\n");
        }
        output.push('\n');
        output.push_str(&extracted);

        Ok(output)
    }
}

fn format_metadata_only(
    path: &str,
    size_bytes: usize,
    format: DetectedFormat,
    message: Option<&str>,
) -> String {
    let mut output = String::new();
    output.push_str(&format!("File: {path}\n"));
    output.push_str(&format!("Size: {size_bytes} bytes\n"));
    output.push_str(&format!("Format: {} (unsupported)\n", format.as_str()));
    output.push_str("Encoding: unknown\n");
    if let Some(msg) = message {
        output.push_str(&format!("\n{msg}. Use 'read' tool for raw access.\n"));
    } else {
        output.push_str(
            "\nUnsupported format for text extraction. Use 'read' tool for raw access.\n",
        );
    }
    output
}

#[async_trait]
impl AgentTool for DocumentExtractTool {
    fn name(&self) -> &str {
        "document_extract"
    }

    fn description(&self) -> &str {
        "Extract text content from local document files (text, HTML, JSON, CSV, Markdown, XML). Read-only, bounded output."
    }

    fn parameters(&self) -> Value {
        tool_parameters!(DocumentExtractInput)
    }

    async fn execute(&self, input: Value) -> ToolResult {
        match self.execute_inner(input).await {
            Ok(content) => ToolResult::success(content),
            Err(e) => ToolResult::error(e.to_string()),
        }
    }

    fn is_read_only(&self) -> bool {
        true
    }

    fn nature(&self) -> ToolNature {
        ToolNature::Read
    }

    fn family(&self) -> ToolFamily {
        ToolFamily::File
    }

    fn is_always_on(&self) -> bool {
        true
    }

    fn permission_profile(&self, input: &Value) -> Vec<ToolPermissionFacet> {
        let path = input
            .get("path")
            .and_then(Value::as_str)
            .map(|s| s.to_string());
        let mut facets = vec![ToolPermissionFacet::new(ToolNature::Read)];
        if let Some(p) = path {
            facets.push(ToolPermissionFacet::with_resource(
                ToolNature::Read,
                p,
                ToolResourceKind::Path,
            ));
        }
        facets
    }

    fn summary_fields(&self) -> &'static [&'static str] {
        &["path", "format"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_id() -> String {
        let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        format!("{}{}", d.as_secs(), d.subsec_nanos())
    }

    fn create_temp_with_ext(content: &str, ext: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("test_{}.{}", unique_id(), ext));
        std::fs::write(&path, content).unwrap();
        path
    }

    fn create_temp_bytes_with_ext(bytes: &[u8], ext: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("test_{}.{}", unique_id(), ext));
        std::fs::write(&path, bytes).unwrap();
        path
    }

    fn run_extract(path: &Path, format_hint: Option<&str>, max_bytes: Option<usize>) -> String {
        let tool = DocumentExtractTool::new(PathBuf::from("/"));
        let input = if let Some(mb) = max_bytes {
            serde_json::json!({
                "path": path.to_string_lossy(),
                "format": format_hint.unwrap_or("auto"),
                "max_bytes": mb,
            })
        } else {
            serde_json::json!({
                "path": path.to_string_lossy(),
                "format": format_hint.unwrap_or("auto"),
            })
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(tool.execute(input));
        assert!(
            !result.is_error,
            "Expected success, got: {}",
            result.content
        );
        result.content
    }

    #[test]
    fn test_extract_plain_text() {
        let content = "Hello, world!\nThis is a test file.\n";
        let path = create_temp_with_ext(content, "txt");
        let output = run_extract(&path, None, None);
        assert!(output.contains("Hello, world!"));
        assert!(output.contains("Format: text"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_extract_markdown_strips_frontmatter() {
        let content = "---\ntitle: Test\ndate: 2026-01-01\n---\n\n# Hello\n\nBody content here.\n";
        let path = create_temp_with_ext(content, "md");
        let output = run_extract(&path, None, None);
        assert!(output.contains("Body content here"));
        assert!(!output.contains("title: Test"));
        assert!(output.contains("Format: markdown"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_extract_html_strips_tags() {
        let content = "<!DOCTYPE html>\n<html><head><title>Test</title></head>\n<body><h1>Hello</h1><p>World</p></body>\n</html>";
        let path = create_temp_with_ext(content, "html");
        let output = run_extract(&path, None, None);
        assert!(output.contains("Hello"));
        assert!(output.contains("World"));
        assert!(!output.contains("<html>"));
        assert!(!output.contains("<h1>"));
        assert!(output.contains("Format: html"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_extract_json_pretty_prints() {
        let content = r#"{"name":"test","value":42,"items":[1,2,3]}"#;
        let path = create_temp_with_ext(content, "json");
        let output = run_extract(&path, None, None);
        assert!(output.contains(r#""name": "test""#));
        assert!(output.contains(r#""value": 42"#));
        assert!(output.contains("Format: json"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_extract_csv_shows_structure() {
        let content = "name,age,city\nAlice,30,NYC\nBob,25,LA\n";
        let path = create_temp_with_ext(content, "csv");
        let output = run_extract(&path, None, None);
        assert!(output.contains("3 columns"));
        assert!(output.contains("2 data rows"));
        assert!(output.contains("name | age | city"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        assert!(output.contains("Format: csv"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_extract_binary_returns_metadata() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("test_{}.bin", unique_id()));
        std::fs::write(&path, &[0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE]).unwrap();
        let output = run_extract(&path, None, None);
        assert!(output.contains("binary"));
        assert!(output.contains("unsupported"));
        assert!(output.contains("read"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_extract_pdf_returns_metadata_without_dumping_bytes() {
        let path = create_temp_bytes_with_ext(
            b"%PDF-1.7\nSECRET PDF BODY SHOULD NOT BE EXTRACTED\n%%EOF",
            "pdf",
        );
        let output = run_extract(&path, None, None);
        assert!(output.contains("Format: pdf (unsupported)"));
        assert!(output.contains("Unsupported format for text extraction"));
        assert!(!output.contains("SECRET PDF BODY"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_extract_image_magic_returns_metadata_without_dumping_bytes() {
        let path = create_temp_bytes_with_ext(
            b"\x89PNG\r\n\x1a\nSECRET IMAGE BODY SHOULD NOT BE EXTRACTED",
            "dat",
        );
        let output = run_extract(&path, None, None);
        assert!(output.contains("Format: image (unsupported)"));
        assert!(output.contains("Unsupported format for text extraction"));
        assert!(!output.contains("SECRET IMAGE BODY"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_extract_office_extension_returns_metadata_without_dumping_bytes() {
        let path = create_temp_bytes_with_ext(
            b"PK\x03\x04SECRET OFFICE BODY SHOULD NOT BE EXTRACTED",
            "docx",
        );
        let output = run_extract(&path, None, None);
        assert!(output.contains("Format: office (unsupported)"));
        assert!(output.contains("Unsupported format for text extraction"));
        assert!(!output.contains("SECRET OFFICE BODY"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_extract_truncates_large_file() {
        let content = "x".repeat(1000);
        let path = create_temp_with_ext(&content, "txt");
        let output = run_extract(&path, None, Some(100));
        assert!(output.contains("truncated"));
        assert!(output.contains("1000 bytes total"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_extract_oversize_file_returns_metadata_only() {
        assert_eq!(MAX_FILE_SIZE, 10 * 1024 * 1024);
    }

    #[test]
    fn test_extract_nonexistent_file_returns_error() {
        let tool = DocumentExtractTool::new(PathBuf::from("/"));
        let input = serde_json::json!({
            "path": "/nonexistent/path/file.txt",
            "format": "auto",
        });
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(tool.execute(input));
        assert!(result.is_error);
        assert!(result.content.contains("file not found"));
    }

    #[test]
    fn test_extract_format_hint_overrides_detection() {
        let content = "<html><body><p>Forced HTML</p></body></html>";
        let path = create_temp_with_ext(content, "txt");
        let output = run_extract(&path, Some("html"), None);
        assert!(output.contains("Forced HTML"));
        assert!(output.contains("Format: html"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_tool_name() {
        let tool = DocumentExtractTool::new(PathBuf::from("."));
        assert_eq!(tool.name(), "document_extract");
    }

    #[test]
    fn test_tool_is_read_only() {
        let tool = DocumentExtractTool::new(PathBuf::from("."));
        assert!(tool.is_read_only());
    }

    #[test]
    fn test_tool_nature() {
        let tool = DocumentExtractTool::new(PathBuf::from("."));
        assert!(matches!(tool.nature(), ToolNature::Read));
    }

    #[test]
    fn test_tool_family() {
        let tool = DocumentExtractTool::new(PathBuf::from("."));
        assert!(matches!(tool.family(), ToolFamily::File));
    }

    #[test]
    fn test_tool_is_always_on() {
        let tool = DocumentExtractTool::new(PathBuf::from("."));
        assert!(tool.is_always_on());
    }

    #[test]
    fn test_tool_summary_fields() {
        let tool = DocumentExtractTool::new(PathBuf::from("."));
        assert_eq!(tool.summary_fields(), &["path", "format"]);
    }

    #[test]
    fn test_tool_has_description() {
        let tool = DocumentExtractTool::new(PathBuf::from("."));
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_tool_emits_parameters_schema() {
        let tool = DocumentExtractTool::new(PathBuf::from("."));
        let schema = tool.parameters();
        assert!(schema.is_object());
        let props = schema.get("properties");
        assert!(props.is_some());
    }

    #[test]
    fn test_tool_permission_profile() {
        let tool = DocumentExtractTool::new(PathBuf::from("."));
        let input = serde_json::json!({"path": "test.txt"});
        let profile = tool.permission_profile(&input);
        assert!(!profile.is_empty());
        for facet in &profile {
            assert_eq!(facet.nature, ToolNature::Read);
        }
    }

    #[test]
    fn test_detect_format_by_extension() {
        assert_eq!(extension_to_format("txt"), Some(DetectedFormat::Text));
        assert_eq!(extension_to_format("md"), Some(DetectedFormat::Markdown));
        assert_eq!(extension_to_format("html"), Some(DetectedFormat::Html));
        assert_eq!(extension_to_format("json"), Some(DetectedFormat::Json));
        assert_eq!(extension_to_format("csv"), Some(DetectedFormat::Csv));
        assert_eq!(extension_to_format("xml"), Some(DetectedFormat::Xml));
        assert_eq!(
            extension_to_format("jsonl"),
            Some(DetectedFormat::JsonLines)
        );
        assert_eq!(extension_to_format("pdf"), Some(DetectedFormat::Pdf));
        assert_eq!(extension_to_format("docx"), Some(DetectedFormat::Office));
        assert_eq!(extension_to_format("png"), Some(DetectedFormat::Image));
        assert_eq!(extension_to_format("zip"), Some(DetectedFormat::Archive));
        assert_eq!(extension_to_format("bin"), None);
    }

    #[test]
    fn test_detect_format_by_hint() {
        assert_eq!(parse_format_hint("html"), Some(DetectedFormat::Html));
        assert_eq!(parse_format_hint("json"), Some(DetectedFormat::Json));
        assert_eq!(parse_format_hint("pdf"), Some(DetectedFormat::Pdf));
        assert_eq!(parse_format_hint("office"), Some(DetectedFormat::Office));
        assert_eq!(parse_format_hint("image"), Some(DetectedFormat::Image));
        assert_eq!(parse_format_hint("archive"), Some(DetectedFormat::Archive));
        assert_eq!(parse_format_hint("auto"), None);
        assert_eq!(parse_format_hint(""), None);
    }

    #[test]
    fn test_looks_like_html() {
        assert!(looks_like_html(b"<!DOCTYPE html>"));
        assert!(looks_like_html(b"<html><body>"));
        assert!(looks_like_html(b"<HTML>"));
        assert!(!looks_like_html(b"Hello world"));
    }

    #[test]
    fn test_looks_like_json() {
        assert!(looks_like_json(b"{\"key\": \"value\"}"));
        assert!(looks_like_json(b"[1, 2, 3]"));
        assert!(!looks_like_json(b"Hello world"));
    }

    #[test]
    fn test_strip_yaml_frontmatter() {
        let with_fm = "---\ntitle: Test\n---\n\nBody text\n";
        assert_eq!(strip_yaml_frontmatter(with_fm), "\nBody text\n");

        let without_fm = "# No frontmatter\n\nBody text\n";
        assert_eq!(strip_yaml_frontmatter(without_fm), without_fm);

        let no_closing = "---\ntitle: Test\n\nBody text\n";
        assert_eq!(strip_yaml_frontmatter(no_closing), no_closing);
    }

    #[test]
    fn test_bound_content_respects_utf8_boundary() {
        let content = "Hello ".repeat(100) + "\u{4E2D}\u{6587}";
        let (bounded, truncated) = bound_content(&content, 50);
        assert!(truncated);
        assert!(bounded.len() <= 50 + TRUNCATION_MARKER.len() + 50);
        assert!(String::from_utf8(bounded.as_bytes().to_vec()).is_ok());
    }

    #[test]
    fn test_extract_xml_strips_tags() {
        let content = "<?xml version=\"1.0\"?><root><item>Hello</item><item>World</item></root>";
        let path = create_temp_with_ext(content, "xml");
        let output = run_extract(&path, None, None);
        assert!(output.contains("Hello"));
        assert!(output.contains("World"));
        assert!(!output.contains("<item>"));
        assert!(output.contains("Format: xml"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_extract_jsonl_formats_lines() {
        let content = "{\"id\": 1, \"name\": \"Alice\"}\n{\"id\": 2, \"name\": \"Bob\"}\n{\"id\": 3, \"name\": \"Charlie\"}";
        let path = create_temp_with_ext(content, "jsonl");
        let output = run_extract(&path, None, None);
        assert!(output.contains("3 lines"));
        assert!(output.contains("Line 1"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_extract_tsv_detects_tab_delimiter() {
        let content = "name\tage\tcity\nAlice\t30\tNYC\n";
        let path = create_temp_with_ext(content, "tsv");
        let output = run_extract(&path, None, None);
        assert!(output.contains("3 columns"));
        assert!(output.contains("1 data rows"));
        assert!(output.contains("Alice"));
        std::fs::remove_file(&path).ok();
    }
}
