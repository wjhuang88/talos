//! Guest-only language wire contract and owned buffers for ADR-079 ABI version two.
//! This workspace is independently built and is not a dependency of the host binary.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

// Compile existing source-only queries, preserving built-in semantics.
#[path = "../../../../crates/talos-text/src/symbol.rs"]
pub mod symbol;
#[path = "../../../../crates/talos-text/src/symbol_queries.rs"]
pub mod symbol_queries;

/// Existing declaration wire shape shared with built-in queries.
pub type SymbolInfo = Symbol;
/// Existing location wire shape shared with built-in queries.
pub type SourceLocation = Location;

fn guarded<T>(operation: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation))
        .unwrap_or_else(|_| Err("parse failed".into()))
}

fn parse_builtin(language: &str, source: &str) -> Result<arborium::tree_sitter::Tree, String> {
    // No clock in wasm32-unknown-unknown. Host fuel/epoch deadline bound the
    // complete invocation, including C parser execution; never call Instant here.
    let grammar = arborium::get_language(language).ok_or("language not loaded")?;
    let mut parser = arborium::tree_sitter::Parser::new();
    parser
        .set_language(&grammar)
        .map_err(|error| error.to_string())?;
    let tree = parser.parse(source, None).ok_or("parse failed")?;
    let mut cursor = tree.walk();
    let mut depth = 0usize;
    let mut nodes = 0usize;
    loop {
        nodes += 1;
        if depth > 128 || nodes > 1_000_000 {
            return Err("parse budget exceeded".into());
        }
        if cursor.goto_first_child() {
            depth += 1;
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                drop(cursor);
                return Ok(tree);
            }
            depth -= 1;
        }
    }
}

/// Use the existing built-in grammar and semantic capture vocabulary.
pub fn analyze(language: &str, source: &str) -> Result<Analysis, String> {
    guarded(|| {
        let spans = arborium::Highlighter::new()
            .highlight_spans(language, source)
            .map_err(|error| error.to_string())?;
        Ok(Analysis {
            // Arborium owns exact-range priority and nested/partial overlap resolution.
            // Keep pattern_index until its normalizer has flattened the captures.
            spans: arborium_highlight::spans_to_flat_tokens(source, spans)
                .into_iter()
                .map(|s| {
                    (
                        s.start as usize,
                        s.end as usize,
                        arborium_theme::tag_to_name(s.tag)
                            .unwrap_or(s.tag)
                            .to_owned(),
                    )
                })
                .collect(),
        })
    })
}

/// Maximum decoded source and serialized result bytes accepted by these guests.
pub const MAX_SOURCE_BYTES: usize = 256 * 1024;
/// JSON may encode one input byte using a six-byte escape.
pub const MAX_REQUEST_BYTES: usize = MAX_SOURCE_BYTES * 6 + 4096;

/// Source-only request. The optional operation selects the existing symbol JSON protocol.
#[derive(Deserialize)]
pub struct Request {
    /// Canonical language identity.
    pub language: String,
    /// Source supplied by the permission-gated host; guests never open files.
    pub source: String,
    /// Symbol JSON protocol version (distinct from memory ABI version two).
    pub abi_version: Option<u32>,
    /// Missing for a highlight request.
    pub operation: Option<Operation>,
}

/// Existing source query wire contract.
#[derive(Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum Operation {
    /// Find the first definition and lexical references.
    FindSymbol { name: String },
    /// Enumerate source declarations, optionally filtering their syntax kind.
    ListSymbols { kind: Option<String> },
    /// Enumerate import declarations.
    ListImports,
    /// Enumerate lexical references in this source only.
    FindReferences { name: String },
}

/// Byte-based, non-overlapping semantic capture.
pub type Span = (usize, usize, String);

/// One source declaration in the existing consumer shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// Declared name.
    pub name: String,
    /// Syntax kind understood by the symbol tools.
    pub kind: String,
    /// Empty label; the host supplies the trusted caller's path.
    pub file: String,
    /// One-based source line.
    pub line: usize,
}

/// Location in caller-supplied source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Empty until filled by the host.
    pub file: String,
    /// One-based line.
    pub line: usize,
    /// One-based byte column for references; zero for definition compatibility.
    pub column: usize,
}

/// Highlight captures produced by the existing Arborium grammar.
#[derive(Default)]
pub struct Analysis {
    /// Semantic captures.
    pub spans: Vec<Span>,
}

#[cfg(all(test, feature = "python"))]
mod tests {
    #[test]
    fn python_highlighting_produces_valid_wire_spans() {
        for source in [
            "def greet():\n    pass\n",
            "# def fake():\ndef 中文():\n    pass\n",
        ] {
            let request =
                serde_json::to_vec(&serde_json::json!({"language":"python", "source":source}))
                    .expect("request");
            let response: serde_json::Value =
                serde_json::from_slice(&super::respond(&request, "python", |source| {
                    super::analyze("python", source)
                }))
                .expect("response");
            assert!(
                response["Spans"].is_array(),
                "response={response}, raw={:?}",
                super::analyze("python", source).expect("analysis").spans
            );
            let spans = super::analyze("python", source).expect("captures").spans;
            let name = if source.contains("中文") {
                "中文"
            } else {
                "greet"
            };
            assert!(
                spans
                    .iter()
                    .any(|(start, end, capture)| &source[*start..*end] == name
                        && capture == "function"),
                "{spans:?}"
            );
        }
    }

    #[test]
    fn python_nested_string_captures_and_symbols_preserve_source() {
        let source = "# def fake():\ndef greet(name):\n    return f\"hello {name}\"\n";
        let spans = super::analyze("python", source).expect("captures").spans;
        let mut end = 0;
        for (start, stop, _) in &spans {
            assert!(*start >= end && stop >= start);
            assert!(source.is_char_boundary(*start) && source.is_char_boundary(*stop));
            end = *stop;
        }
        assert!(spans.iter().any(|(_, _, capture)| capture == "string"));
        let symbols =
            super::symbol::list_symbols("python", source, "", Some("function")).expect("symbols");
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "greet");
    }
}

/// Execute one bounded source request using a language-specific parser.
pub fn respond(
    bytes: &[u8],
    language: &str,
    analyze: fn(&str) -> Result<Analysis, String>,
) -> Vec<u8> {
    let value = (|| -> Result<Value, String> {
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err("request exceeds guest limit".into());
        }
        let request: Request = serde_json::from_slice(bytes).map_err(|_| "invalid request")?;
        if request.language != language || request.source.len() > MAX_SOURCE_BYTES {
            return Err("unsupported language or oversized source".into());
        }
        if request.operation.is_some() && request.abi_version != Some(1) {
            return Err("unsupported symbol protocol".into());
        }
        let Some(operation) = request.operation else {
            let mut analysis = analyze(&request.source)?;
            analysis.spans.sort_by_key(|span| (span.0, span.1));
            let mut end = 0;
            for (start, stop, _) in &analysis.spans {
                if *start < end
                    || start > stop
                    || !request.source.is_char_boundary(*start)
                    || !request.source.is_char_boundary(*stop)
                {
                    return Err("invalid parser source range".into());
                }
                end = *stop;
            }
            return Ok(json!({"Spans": analysis.spans}));
        };
        let path = std::path::Path::new("");
        let result = match operation {
            Operation::ListSymbols { kind } => json!(symbol::list_symbols(
                language,
                &request.source,
                "",
                kind.as_deref()
            )?),
            Operation::ListImports => json!(symbol_queries::list_imports(
                language,
                &request.source,
                path
            )?),
            Operation::FindReferences { name } => json!(symbol_queries::find_references(
                language,
                &request.source,
                path,
                &name
            )?),
            Operation::FindSymbol { name } => json!(symbol_queries::find_symbol(
                language,
                &request.source,
                path,
                path,
                &name
            )),
        };
        Ok(json!({"Result": result}))
    })();
    let response = value.unwrap_or_else(|reason| json!({"Unavailable": reason}));
    let encoded = serde_json::to_vec(&response).unwrap_or_default();
    if encoded.len() > MAX_SOURCE_BYTES {
        br#"{"Unavailable":"response exceeds guest limit"}"#.to_vec()
    } else {
        encoded
    }
}

/// One instance's request/response ownership. No arbitrary pointer is ever dereferenced.
#[derive(Default)]
pub struct Buffers {
    input: Vec<u8>,
    output: Vec<u8>,
    allocated: bool,
    consumed: bool,
}

impl Buffers {
    /// Allocate once and expose initialized bytes while the guest is stopped.
    pub fn allocate(&mut self, length: i32) -> i32 {
        if self.allocated || length <= 0 || length as usize > MAX_REQUEST_BYTES {
            return 0;
        }
        self.input = vec![0; length as usize];
        self.allocated = true;
        self.input.as_mut_ptr() as usize as i32
    }

    /// Validate exact retained ownership before parsing; keep output alive until instance disposal.
    pub fn run(
        &mut self,
        pointer: i32,
        length: i32,
        language: &str,
        analyze: fn(&str) -> Result<Analysis, String>,
    ) -> i64 {
        if !self.allocated
            || self.consumed
            || length < 0
            || length as usize != self.input.len()
            || pointer != self.input.as_ptr() as usize as i32
        {
            return 0;
        }
        self.consumed = true;
        self.output = respond(&self.input, language, analyze);
        ((self.output.as_ptr() as u64) << 32 | self.output.len() as u64) as i64
    }
}

/// Define an isolated WASM guest's ABI exports. Native unit tests expose no ABI symbols.
#[macro_export]
macro_rules! export_provider {
    ($language:literal, $analyze:path) => {
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        mod guest_abi {
            std::thread_local! {
                static BUFFERS: std::cell::RefCell<$crate::Buffers> = std::cell::RefCell::default();
            }
            // SAFETY: ADR-079 permits only these unique symbols in standalone wasm32 guests.
            // No imports/reentrancy; every RefCell borrow ends before host memory access resumes.
            #[unsafe(export_name = "talos_language_abi_version")]
            pub extern "C" fn version() -> i32 {
                2
            }
            // SAFETY: unique ABI-v2 (i32)->i32 export, confined to this standalone WASM module.
            #[unsafe(export_name = "talos_language_alloc")]
            pub extern "C" fn allocate(length: i32) -> i32 {
                BUFFERS.with(|buffers| buffers.borrow_mut().allocate(length))
            }
            // SAFETY: unique ABI-v2 (i32,i32)->i64 export; owned-buffer checks reject forged input.
            #[unsafe(export_name = "talos_language_run")]
            pub extern "C" fn run(pointer: i32, length: i32) -> i64 {
                BUFFERS.with(|buffers| {
                    buffers
                        .borrow_mut()
                        .run(pointer, length, $language, $analyze)
                })
            }
        }
    };
}
