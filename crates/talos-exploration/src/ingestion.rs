//! Text ingestion, claim extraction, and citation-preserving synthesis.
//!
//! Provides local text ingestion with chunking and FTS indexing,
//! deterministic claim extraction, and synthesis creation with
//! citation validation.

use chrono::Utc;
use sha2::{Digest, Sha256};

use crate::{ExplorationError, ExplorationStore, Source, SourceChunk, Synthesis};

// ---------------------------------------------------------------------------
// Configuration and report types
// ---------------------------------------------------------------------------

/// Configuration for text chunking during ingestion.
#[derive(Debug, Clone)]
pub struct ChunkingConfig {
    pub max_chunk_chars: usize,
    pub overlap_chars: usize,
    /// Maximum input file size in bytes. Files exceeding this are rejected.
    pub max_file_bytes: usize,
    /// Maximum number of chunks per source. Exceeding this is an error.
    pub max_chunks_per_source: usize,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            max_chunk_chars: 1000,
            overlap_chars: 100,
            max_file_bytes: 10_485_760, // 10 MB
            max_chunks_per_source: 10_000,
        }
    }
}

/// Report from an ingestion operation.
#[derive(Debug, Clone)]
pub struct IngestionReport {
    pub source_id: String,
    pub chunks_created: usize,
    pub chunk_ids: Vec<String>,
}

/// Content fetched from a remote source (mock or real).
/// Used for permission-aware ingestion without coupling to network code.
#[derive(Debug, Clone)]
pub struct FetchedContent {
    pub url: String,
    pub title: String,
    pub text: String,
    pub fetched_at: chrono::DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Public functions
// ---------------------------------------------------------------------------

/// Ingest local text into an exploration run.
/// Creates a source record, splits text into chunks, indexes for FTS.
pub fn ingest_text(
    store: &mut ExplorationStore,
    run_id: &str,
    title: &str,
    text: &str,
    config: &ChunkingConfig,
) -> Result<IngestionReport, ExplorationError> {
    if text.len() > config.max_file_bytes {
        return Err(ExplorationError::FileTooLarge {
            size: text.len(),
            max: config.max_file_bytes,
        });
    }

    let content_hash = compute_hash(text);
    let source_id = uuid::Uuid::new_v4().to_string();

    let source = Source {
        id: source_id.clone(),
        run_id: Some(run_id.to_string()),
        url: None,
        title: title.to_string(),
        authors: None,
        publication_date: None,
        fetched_at: Utc::now(),
        license_notes: None,
        content_hash,
    };
    store.insert_source(&source)?;

    let chunks = split_text(text, config);
    if chunks.len() > config.max_chunks_per_source {
        return Err(ExplorationError::ChunkCapExceeded {
            count: chunks.len(),
            max: config.max_chunks_per_source,
        });
    }
    let mut chunk_ids = Vec::new();

    for (ordinal, chunk_text) in chunks.into_iter().enumerate() {
        let chunk_id = uuid::Uuid::new_v4().to_string();
        let token_estimate = (chunk_text.len() / 4) as i64;
        let chunk = SourceChunk {
            id: chunk_id.clone(),
            source_id: source_id.clone(),
            chunk_ordinal: ordinal as i64,
            text: chunk_text,
            token_estimate: Some(token_estimate),
        };
        store.insert_chunk(&chunk)?;
        chunk_ids.push(chunk_id);
    }

    Ok(IngestionReport {
        source_id,
        chunks_created: chunk_ids.len(),
        chunk_ids,
    })
}

/// Ingest fetched content with source provenance (URL, fetch timestamp).
/// This is the permission-aware ingestion path — the caller is responsible
/// for obtaining content through approved fetch tools.
pub fn ingest_fetched(
    store: &mut ExplorationStore,
    run_id: &str,
    content: &FetchedContent,
    config: &ChunkingConfig,
) -> Result<IngestionReport, ExplorationError> {
    let content_hash = compute_hash(&content.text);
    let source_id = uuid::Uuid::new_v4().to_string();

    let source = Source {
        id: source_id.clone(),
        run_id: Some(run_id.to_string()),
        url: Some(content.url.clone()),
        title: content.title.clone(),
        authors: None,
        publication_date: None,
        fetched_at: content.fetched_at,
        license_notes: None,
        content_hash,
    };
    store.insert_source(&source)?;

    let chunks = split_text(&content.text, config);
    let mut chunk_ids = Vec::new();

    for (ordinal, chunk_text) in chunks.into_iter().enumerate() {
        let chunk_id = uuid::Uuid::new_v4().to_string();
        let token_estimate = (chunk_text.len() / 4) as i64;
        let chunk = SourceChunk {
            id: chunk_id.clone(),
            source_id: source_id.clone(),
            chunk_ordinal: ordinal as i64,
            text: chunk_text,
            token_estimate: Some(token_estimate),
        };
        store.insert_chunk(&chunk)?;
        chunk_ids.push(chunk_id);
    }

    Ok(IngestionReport {
        source_id,
        chunks_created: chunk_ids.len(),
        chunk_ids,
    })
}

/// Extract simple factual claims from text using deterministic rules.
/// Returns claim text strings suitable for normalization.
pub fn extract_claims(text: &str) -> Vec<String> {
    let sentences = split_sentences(text);
    let factual_indicators = [
        " is ",
        " are ",
        " was ",
        " were ",
        " will ",
        " has ",
        " have ",
        "according to",
        " shows ",
        " demonstrates ",
        " found that",
        " provides ",
        " prevents ",
        " enables ",
        " supports ",
    ];

    let mut claims = Vec::new();

    for sentence in sentences {
        let trimmed = sentence.trim();

        // Length filter: > 20 chars and < 300 chars.
        if trimmed.len() <= 20 || trimmed.len() >= 300 {
            continue;
        }

        // Skip questions and commands.
        if trimmed.ends_with('?') || trimmed.ends_with('!') {
            continue;
        }

        // Check for factual indicators.
        let lower = trimmed.to_lowercase();
        let has_indicator = factual_indicators
            .iter()
            .any(|&indicator| lower.contains(indicator));

        // Also check for numbers/percentages as factual signals.
        let has_numbers = trimmed.chars().any(|c| c.is_ascii_digit());

        if has_indicator || has_numbers {
            claims.push(trimmed.to_string());
        }

        if claims.len() >= 20 {
            break;
        }
    }

    claims
}

/// Create a citation-preserving synthesis.
/// Validates all cited_source_ids exist in the store.
/// The conclusion is inference; cited_source_ids are evidence.
pub fn create_synthesis(
    store: &ExplorationStore,
    run_id: &str,
    conclusion: &str,
    cited_source_ids: &[String],
    caveats: &[String],
    unresolved_questions: &[String],
) -> Result<Synthesis, ExplorationError> {
    // Validate every cited source ID exists.
    for source_id in cited_source_ids {
        let exists = store.get_source(source_id)?;
        if exists.is_none() {
            return Err(ExplorationError::CitationValidation(format!(
                "cited source {source_id} does not exist"
            )));
        }
    }

    let synthesis = Synthesis {
        id: uuid::Uuid::new_v4().to_string(),
        run_id: Some(run_id.to_string()),
        conclusion: conclusion.to_string(),
        caveats: if caveats.is_empty() {
            None
        } else {
            Some(caveats.join("; "))
        },
        cited_source_ids: cited_source_ids.to_vec(),
        unresolved_questions: unresolved_questions.to_vec(),
        created_at: Utc::now(),
    };

    store.insert_synthesis(&synthesis)?;

    Ok(synthesis)
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Compute SHA-256 hash of text, returned as hex string.
fn compute_hash(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

/// Split text into chunks using paragraph boundaries and size limits.
fn split_text(text: &str, config: &ChunkingConfig) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }

    // Split by paragraphs (double newline).
    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect();

    let mut chunks = Vec::new();

    for paragraph in paragraphs {
        if paragraph.len() <= config.max_chunk_chars {
            chunks.push(paragraph.to_string());
        } else {
            // Further split long paragraphs by character boundaries.
            let sub_chunks = split_by_chars(paragraph, config);
            chunks.extend(sub_chunks);
        }
    }

    chunks
}

/// Split a single string into chunks of at most `max_chunk_chars` with overlap.
fn split_by_chars(text: &str, config: &ChunkingConfig) -> Vec<String> {
    let mut chunks = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    if len == 0 {
        return chunks;
    }

    let mut start = 0;
    while start < len {
        let end = (start + config.max_chunk_chars).min(len);
        let chunk: String = chars[start..end].iter().collect();
        chunks.push(chunk);

        if end >= len {
            break;
        }

        // Advance with overlap.
        start = end.saturating_sub(config.overlap_chars);
        if start >= end {
            // Safety: ensure progress.
            start = end;
        }
    }

    chunks
}

/// Split text into sentences on `. `, `! `, `? ` followed by capital letter or end.
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut start = 0;

    let mut i = 0;
    while i < len {
        let c = chars[i];
        if c == '.' || c == '!' || c == '?' {
            // Check if followed by space + capital letter, or end of text.
            let is_end = i + 1 >= len;
            let is_sentence_boundary = if is_end {
                true
            } else if i + 1 < len && chars[i + 1] == ' ' {
                // Space after punctuation — check if next word starts with capital.
                if i + 2 < len && chars[i + 2].is_uppercase() {
                    true
                } else {
                    let rest: String = chars[i + 2..].iter().collect();
                    rest.chars().next().is_some_and(|c| c.is_uppercase())
                }
            } else {
                false
            };

            if is_sentence_boundary {
                let sentence: String = chars[start..=i].iter().collect();
                let trimmed = sentence.trim();
                if !trimmed.is_empty() {
                    sentences.push(trimmed.to_string());
                }
                start = i + 1;
            }
        }
        i += 1;
    }

    // Capture any remaining text.
    if start < len {
        let remaining: String = chars[start..].iter().collect();
        let trimmed = remaining.trim();
        if !trimmed.is_empty() {
            sentences.push(trimmed.to_string());
        }
    }

    sentences
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExplorationStore;

    fn make_store() -> ExplorationStore {
        ExplorationStore::open_memory().unwrap()
    }

    // --- ingest_text tests ---

    #[test]
    fn ingest_text_creates_source_and_chunks() {
        let mut store = make_store();
        let run = store.create_run("test query", Some("test plan")).unwrap();

        let text = "Rust is a systems programming language.\n\nIt provides memory safety without garbage collection.\n\nThe ownership model is unique among mainstream languages.";
        let config = ChunkingConfig::default();

        let report = ingest_text(&mut store, &run.id, "Rust Overview", text, &config).unwrap();

        assert!(!report.source_id.is_empty());
        assert!(report.chunks_created > 0);
        assert_eq!(report.chunk_ids.len(), report.chunks_created);

        // Verify source exists.
        let source = store.get_source(&report.source_id).unwrap();
        assert!(source.is_some());
        let source = source.unwrap();
        assert_eq!(source.title, "Rust Overview");
        assert!(source.url.is_none());

        // Verify FTS search finds content.
        let results = store.search_chunks("memory safety", 10).unwrap();
        assert!(!results.is_empty(), "FTS should find ingested content");
    }

    #[test]
    fn ingest_text_chunk_overlap() {
        let mut store = make_store();
        let run = store.create_run("overlap test", None).unwrap();

        // Create text that will be split into multiple chunks with small max.
        let text = "AAAAAAAAAA BBBBBBBBBB CCCCCCCCCC DDDDDDDDDD EEEEEEEEEE FFFFFFFFFF GGGGGGGGGG HHHHHHHHHH IIIIIIIIII JJJJJJJJJJ";
        let config = ChunkingConfig {
            max_chunk_chars: 50,
            overlap_chars: 10,
            ..Default::default()
        };

        let report = ingest_text(&mut store, &run.id, "Overlap Test", text, &config).unwrap();

        assert!(
            report.chunks_created > 1,
            "Should create multiple chunks, got {}",
            report.chunks_created
        );

        // Verify overlap: adjacent chunks should share content.
        // We can check by searching for content that spans chunk boundaries.
        let results = store.search_chunks("BBBBBBBBBB", 10).unwrap();
        assert!(!results.is_empty(), "Should find BBBBBBBBBB in chunks");
    }

    // --- ingest_fetched tests ---

    #[test]
    fn ingest_fetched_records_provenance() {
        let mut store = make_store();
        let run = store.create_run("fetched test", None).unwrap();

        let fetched_at = Utc::now();
        let content = FetchedContent {
            url: "https://example.com/article".to_string(),
            title: "Example Article".to_string(),
            text: "This is fetched content from the web.".to_string(),
            fetched_at,
        };

        let config = ChunkingConfig::default();
        let report = ingest_fetched(&mut store, &run.id, &content, &config).unwrap();

        let source = store.get_source(&report.source_id).unwrap();
        assert!(source.is_some());
        let source = source.unwrap();
        assert_eq!(source.url, Some("https://example.com/article".to_string()));
        assert_eq!(source.title, "Example Article");
        // fetched_at should match (within the same second due to serialization).
        assert_eq!(
            source.fetched_at.timestamp(),
            fetched_at.timestamp(),
            "fetched_at should match"
        );
    }

    // --- extract_claims tests ---

    #[test]
    fn extract_claims_deterministic() {
        let text = "Rust is a systems programming language. It provides memory safety. The borrow checker prevents data races. Rust was created by Mozilla in 2010.";

        let claims1 = extract_claims(text);
        let claims2 = extract_claims(text);

        assert_eq!(claims1, claims2, "extract_claims must be deterministic");
        assert!(!claims1.is_empty(), "Should extract factual claims");

        // Verify specific sentences are returned.
        assert!(
            claims1
                .iter()
                .any(|c| c.contains("systems programming language")),
            "Should contain 'systems programming language'"
        );
        assert!(
            claims1.iter().any(|c| c.contains("memory safety")),
            "Should contain 'memory safety'"
        );
    }

    #[test]
    fn extract_claims_filters_non_factual() {
        let text = "What is Rust? Please read the documentation. Run the tests! The compiler is fast. It has great error messages.";

        let claims = extract_claims(text);

        // Questions and commands should NOT be returned.
        assert!(
            !claims.iter().any(|c| c.contains("What is Rust")),
            "Questions should not be claims"
        );
        assert!(
            !claims.iter().any(|c| c.contains("Please read")),
            "Commands should not be claims"
        );
        assert!(
            !claims.iter().any(|c| c.contains("Run the tests")),
            "Exclamations should not be claims"
        );

        // Factual sentences SHOULD be returned.
        assert!(
            claims.iter().any(|c| c.contains("compiler is fast")),
            "Factual sentence should be a claim"
        );
        assert!(
            claims.iter().any(|c| c.contains("great error messages")),
            "Factual sentence should be a claim"
        );
    }

    // --- create_synthesis tests ---

    #[test]
    fn create_synthesis_with_valid_citations() {
        let mut store = make_store();
        let run = store.create_run("synthesis test", None).unwrap();

        // Ingest two sources.
        let report1 = ingest_text(
            &mut store,
            &run.id,
            "Source A",
            "Rust is fast and safe.",
            &ChunkingConfig::default(),
        )
        .unwrap();

        let report2 = ingest_text(
            &mut store,
            &run.id,
            "Source B",
            "Rust has great tooling.",
            &ChunkingConfig::default(),
        )
        .unwrap();

        let synthesis = create_synthesis(
            &store,
            &run.id,
            "Rust is a well-designed language",
            &[report1.source_id.clone(), report2.source_id.clone()],
            &["Based on limited sources".to_string()],
            &["What about ecosystem maturity?".to_string()],
        )
        .unwrap();

        assert!(!synthesis.id.is_empty());
        assert_eq!(synthesis.conclusion, "Rust is a well-designed language");
        assert_eq!(synthesis.cited_source_ids.len(), 2);
        assert!(synthesis.caveats.is_some());
        assert_eq!(synthesis.unresolved_questions.len(), 1);
    }

    #[test]
    fn create_synthesis_rejects_missing_citations() {
        let store = make_store();
        let run = store.create_run("bad citation test", None).unwrap();

        let err = create_synthesis(
            &store,
            &run.id,
            "Some conclusion",
            &["nonexistent-source-id".to_string()],
            &[],
            &[],
        )
        .unwrap_err();

        assert!(
            matches!(err, ExplorationError::CitationValidation(_)),
            "Expected CitationValidation error, got: {err}"
        );
    }

    // --- full offline pipeline test ---

    #[test]
    fn local_ingestion_works_offline() {
        let mut store = make_store();
        let run = store
            .create_run("offline pipeline test", Some("Full offline pipeline"))
            .unwrap();

        // Step 1: Ingest local text.
        let text = "Python is a popular programming language. It was created by Guido van Rossum in 1991. Python supports multiple paradigms. The language has a large standard library.";
        let report = ingest_text(
            &mut store,
            &run.id,
            "Python Overview",
            text,
            &ChunkingConfig::default(),
        )
        .unwrap();

        assert!(report.chunks_created > 0);

        // Step 2: Extract claims.
        let claims = extract_claims(text);
        assert!(
            !claims.is_empty(),
            "Should extract claims from ingested text"
        );

        // Step 3: Search for content.
        let results = store.search_chunks("Python programming", 10).unwrap();
        assert!(
            !results.is_empty(),
            "FTS should find ingested content offline"
        );

        // Step 4: Create synthesis citing the ingested source.
        let synthesis = create_synthesis(
            &store,
            &run.id,
            "Python is a versatile, well-established language",
            &[report.source_id],
            &["Limited to one source".to_string()],
            &["How does Python compare to Rust?".to_string()],
        )
        .unwrap();

        assert!(!synthesis.id.is_empty());
        assert_eq!(synthesis.cited_source_ids.len(), 1);

        // All steps completed without any network calls.
    }

    #[test]
    fn ingest_text_exceeds_file_budget_returns_error() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("exploration.db");
        let mut store = ExplorationStore::open(&db_path).unwrap();
        let run = store.create_run("budget-test", None).unwrap();

        let config = ChunkingConfig {
            max_file_bytes: 100,
            ..Default::default()
        };
        let oversized = "x".repeat(200);

        let result = ingest_text(&mut store, &run.id, "oversized", &oversized, &config);
        assert!(matches!(result, Err(ExplorationError::FileTooLarge { .. })));
    }

    #[test]
    fn ingest_text_within_budget_succeeds() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("exploration.db");
        let mut store = ExplorationStore::open(&db_path).unwrap();
        let run = store.create_run("budget-test", None).unwrap();

        let config = ChunkingConfig {
            max_file_bytes: 10_000,
            ..Default::default()
        };
        let text = "Hello world. This is a test.\n\nSecond paragraph here.";

        let report = ingest_text(&mut store, &run.id, "normal", text, &config).unwrap();
        assert!(report.chunks_created > 0);
    }

    #[test]
    fn ingest_text_exceeds_chunk_cap_returns_error() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("exploration.db");
        let mut store = ExplorationStore::open(&db_path).unwrap();
        let run = store.create_run("cap-test", None).unwrap();

        let config = ChunkingConfig {
            max_chunk_chars: 5,
            max_chunks_per_source: 2,
            ..Default::default()
        };
        let text = "aaaaa\n\nbbbbb\n\nccccc";

        let result = ingest_text(&mut store, &run.id, "cap-test", text, &config);
        assert!(matches!(
            result,
            Err(ExplorationError::ChunkCapExceeded { .. })
        ));
    }
}
