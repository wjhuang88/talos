use crate::error::CatalogError;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;
use talos_core::model::{
    ModelCapabilities, ModelMetadata, ModelPricing, ModelSource, ProviderInfo, ProviderSource,
};

const SCHEMA_VERSION: u32 = 1;

/// SQLite-backed model catalog store.
///
/// Wraps a `rusqlite::Connection` to provide durable storage for provider
/// metadata, model metadata, and pricing. All methods propagate errors via
/// `Result`; callers should degrade to built-in TOML data on failure.
pub struct ModelCatalog {
    conn: Connection,
}

impl ModelCatalog {
    /// Opens or creates the catalog database at the given path.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, CatalogError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        Self::configure_and_migrate(conn)
    }

    /// Opens an in-memory catalog for testing.
    pub fn open_memory() -> Result<Self, CatalogError> {
        let conn = Connection::open_in_memory()?;
        Self::configure_and_migrate(conn)
    }

    fn configure_and_migrate(conn: Connection) -> Result<Self, CatalogError> {
        conn.execute_batch(
            "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;",
        )?;
        let catalog = Self { conn };
        catalog.migrate()?;
        Ok(catalog)
    }

    fn migrate(&self) -> Result<(), CatalogError> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY
            );

            CREATE TABLE IF NOT EXISTS providers (
                id           TEXT PRIMARY KEY,
                name         TEXT NOT NULL,
                api_base_url TEXT,
                env_var      TEXT,
                npm_package  TEXT,
                doc_url      TEXT,
                source       TEXT NOT NULL DEFAULT 'builtin'
            );

            CREATE TABLE IF NOT EXISTS models (
                id                TEXT NOT NULL,
                provider          TEXT NOT NULL,
                name              TEXT,
                context_limit     INTEGER,
                output_limit      INTEGER,
                reasoning         INTEGER DEFAULT 0,
                tool_call         INTEGER DEFAULT 0,
                structured_output INTEGER DEFAULT 0,
                attachment        INTEGER DEFAULT 0,
                release_date      TEXT,
                source            TEXT NOT NULL DEFAULT 'builtin',
                PRIMARY KEY (provider, id),
                FOREIGN KEY (provider) REFERENCES providers(id)
            );

            CREATE TABLE IF NOT EXISTS pricing (
                model_id           TEXT NOT NULL,
                provider           TEXT NOT NULL,
                input_per_1m       REAL,
                output_per_1m      REAL,
                cache_read_per_1m  REAL,
                cache_write_per_1m REAL,
                PRIMARY KEY (provider, model_id),
                FOREIGN KEY (provider, model_id) REFERENCES models(provider, id)
            );

            CREATE TABLE IF NOT EXISTS catalog_meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )?;

        let version_count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM schema_version", [], |row| row.get(0))?;
        if version_count == 0 {
            self.conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                params![SCHEMA_VERSION],
            )?;
        }

        let stored: i64 =
            self.conn
                .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
                    row.get(0)
                })?;
        if stored as u32 != SCHEMA_VERSION {
            return Err(CatalogError::IncompatibleSchema {
                expected: SCHEMA_VERSION,
                found: stored as u32,
            });
        }

        Ok(())
    }

    pub fn schema_version(&self) -> Result<u32, CatalogError> {
        let version: i64 =
            self.conn
                .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
                    row.get(0)
                })?;
        Ok(version as u32)
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<(), CatalogError> {
        self.conn.execute(
            "INSERT INTO catalog_meta (key, value) VALUES (?1, ?2) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> Result<Option<String>, CatalogError> {
        let value: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM catalog_meta WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?;
        Ok(value)
    }

    pub fn last_refreshed(&self) -> Result<Option<String>, CatalogError> {
        self.get_meta("last_refreshed")
    }

    pub fn model_count(&self) -> Result<usize, CatalogError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM models", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn provider_count(&self) -> Result<usize, CatalogError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM providers", [], |row| row.get(0))?;
        Ok(count as usize)
    }

    pub fn upsert_provider(&self, info: &ProviderInfo) -> Result<(), CatalogError> {
        let source = source_to_str(&info.source);
        self.conn.execute(
            "INSERT INTO providers (id, name, api_base_url, env_var, npm_package, doc_url, source) \
             VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6) \
             ON CONFLICT(id) DO UPDATE SET \
             name = excluded.name, api_base_url = excluded.api_base_url, \
             env_var = excluded.env_var, doc_url = excluded.doc_url, source = excluded.source",
            params![
                info.id,
                info.name,
                info.api_base_url,
                info.env_var,
                info.doc_url,
                source
            ],
        )?;
        Ok(())
    }

    pub fn upsert_model(&self, model: &ModelMetadata) -> Result<(), CatalogError> {
        let source = model_source_to_str(&model.source);
        let tx = self.conn.unchecked_transaction()?;

        tx.execute(
            "INSERT INTO models (id, provider, name, context_limit, output_limit, \
             reasoning, tool_call, structured_output, attachment, release_date, source) \
             VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10) \
             ON CONFLICT(provider, id) DO UPDATE SET \
             context_limit = excluded.context_limit, output_limit = excluded.output_limit, \
             reasoning = excluded.reasoning, tool_call = excluded.tool_call, \
             structured_output = excluded.structured_output, attachment = excluded.attachment, \
             release_date = excluded.release_date, source = excluded.source",
            params![
                model.id,
                model.provider,
                model.context_limit,
                model.output_limit,
                model.capabilities.reasoning,
                model.capabilities.tools,
                model.capabilities.structured_output,
                model.capabilities.image_input,
                model.release_date,
                source,
            ],
        )?;

        tx.execute(
            "DELETE FROM pricing WHERE provider = ?1 AND model_id = ?2",
            params![model.provider, model.id],
        )?;

        if let Some(ref pricing) = model.pricing {
            tx.execute(
                "INSERT INTO pricing (model_id, provider, input_per_1m, output_per_1m, \
                 cache_read_per_1m, cache_write_per_1m) \
                 VALUES (?1, ?2, ?3, ?4, ?5, NULL)",
                params![
                    model.id,
                    model.provider,
                    pricing.input_per_1m,
                    pricing.output_per_1m,
                    pricing.cache_read_per_1m,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn seed(
        &self,
        providers: &[ProviderInfo],
        models: &[ModelMetadata],
        refreshed_at: &str,
    ) -> Result<(), CatalogError> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute_batch("DELETE FROM pricing; DELETE FROM models; DELETE FROM providers;")?;

        for info in providers {
            let source = source_to_str(&info.source);
            tx.execute(
                "INSERT INTO providers (id, name, api_base_url, env_var, npm_package, doc_url, source) \
                 VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6) \
                 ON CONFLICT(id) DO UPDATE SET \
                 name = excluded.name, api_base_url = excluded.api_base_url, \
                 env_var = excluded.env_var, doc_url = excluded.doc_url, source = excluded.source",
                params![info.id, info.name, info.api_base_url, info.env_var, info.doc_url, source],
            )?;
        }

        let mut known_provider_ids: std::collections::HashSet<&str> =
            providers.iter().map(|p| p.id.as_str()).collect();

        for model in models {
            if !known_provider_ids.contains(model.provider.as_str()) {
                let source = model_source_to_str(&model.source);
                tx.execute(
                    "INSERT INTO providers (id, name, api_base_url, env_var, npm_package, doc_url, source) \
                     VALUES (?1, ?1, NULL, NULL, NULL, NULL, ?2) \
                     ON CONFLICT(id) DO NOTHING",
                    params![model.provider, source],
                )?;
                known_provider_ids.insert(model.provider.as_str());
            }

            tx.execute(
                "INSERT INTO models (id, provider, name, context_limit, output_limit, \
                 reasoning, tool_call, structured_output, attachment, release_date, source) \
                 VALUES (?1, ?2, NULL, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    model.id,
                    model.provider,
                    model.context_limit,
                    model.output_limit,
                    model.capabilities.reasoning,
                    model.capabilities.tools,
                    model.capabilities.structured_output,
                    model.capabilities.image_input,
                    model.release_date,
                    model_source_to_str(&model.source),
                ],
            )?;

            if let Some(ref pricing) = model.pricing {
                tx.execute(
                    "INSERT INTO pricing (model_id, provider, input_per_1m, output_per_1m, \
                     cache_read_per_1m, cache_write_per_1m) \
                     VALUES (?1, ?2, ?3, ?4, ?5, NULL)",
                    params![
                        model.id,
                        model.provider,
                        pricing.input_per_1m,
                        pricing.output_per_1m,
                        pricing.cache_read_per_1m,
                    ],
                )?;
            }
        }

        tx.execute(
            "INSERT INTO catalog_meta (key, value) VALUES ('last_refreshed', ?1) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![refreshed_at],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn all_models(&self) -> Result<Vec<ModelMetadata>, CatalogError> {
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.provider, m.context_limit, m.output_limit, \
             m.reasoning, m.tool_call, m.structured_output, m.attachment, \
             m.release_date, m.source, \
             p.input_per_1m, p.output_per_1m, p.cache_read_per_1m \
             FROM models m \
             LEFT JOIN pricing p ON p.provider = m.provider AND p.model_id = m.id \
             ORDER BY m.provider, m.id",
        )?;
        let rows = stmt.query_map([], row_to_model)?;
        let mut models = Vec::new();
        for row in rows {
            models.push(row?);
        }
        Ok(models)
    }

    pub fn models_by_provider(&self, provider: &str) -> Result<Vec<ModelMetadata>, CatalogError> {
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.provider, m.context_limit, m.output_limit, \
             m.reasoning, m.tool_call, m.structured_output, m.attachment, \
             m.release_date, m.source, \
             p.input_per_1m, p.output_per_1m, p.cache_read_per_1m \
             FROM models m \
             LEFT JOIN pricing p ON p.provider = m.provider AND p.model_id = m.id \
             WHERE m.provider = ?1 \
             ORDER BY m.id",
        )?;
        let rows = stmt.query_map(params![provider], row_to_model)?;
        let mut models = Vec::new();
        for row in rows {
            models.push(row?);
        }
        Ok(models)
    }

    pub fn all_providers(&self) -> Result<Vec<ProviderInfo>, CatalogError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, api_base_url, env_var, doc_url, source FROM providers ORDER BY id",
        )?;
        let rows = stmt.query_map([], row_to_provider)?;
        let mut providers = Vec::new();
        for row in rows {
            providers.push(row?);
        }
        Ok(providers)
    }

    pub fn find_model(
        &self,
        provider: &str,
        model_id: &str,
    ) -> Result<Option<ModelMetadata>, CatalogError> {
        let result = self
            .conn
            .query_row(
                "SELECT m.id, m.provider, m.context_limit, m.output_limit, \
                 m.reasoning, m.tool_call, m.structured_output, m.attachment, \
                 m.release_date, m.source, \
                 p.input_per_1m, p.output_per_1m, p.cache_read_per_1m \
                 FROM models m \
                 LEFT JOIN pricing p ON p.provider = m.provider AND p.model_id = m.id \
                 WHERE m.provider = ?1 AND m.id = ?2",
                params![provider, model_id],
                row_to_model,
            )
            .optional()?;
        Ok(result)
    }

    pub fn model_count_for_provider(&self, provider: &str) -> Result<usize, CatalogError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM models WHERE provider = ?1",
            params![provider],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn search_models(&self, query: &str) -> Result<Vec<ModelMetadata>, CatalogError> {
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT m.id, m.provider, m.context_limit, m.output_limit, \
             m.reasoning, m.tool_call, m.structured_output, m.attachment, \
             m.release_date, m.source, \
             p.input_per_1m, p.output_per_1m, p.cache_read_per_1m \
             FROM models m \
             LEFT JOIN pricing p ON p.provider = m.provider AND p.model_id = m.id \
             WHERE m.id LIKE ?1 COLLATE NOCASE OR m.provider LIKE ?1 COLLATE NOCASE \
             ORDER BY m.provider, m.id",
        )?;
        let rows = stmt.query_map(params![pattern], row_to_model)?;
        let mut models = Vec::new();
        for row in rows {
            models.push(row?);
        }
        Ok(models)
    }

    pub fn search_providers(&self, query: &str) -> Result<Vec<ProviderInfo>, CatalogError> {
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT id, name, api_base_url, env_var, doc_url, source \
             FROM providers \
             WHERE id LIKE ?1 COLLATE NOCASE OR name LIKE ?1 COLLATE NOCASE \
             ORDER BY id",
        )?;
        let rows = stmt.query_map(params![pattern], row_to_provider)?;
        let mut providers = Vec::new();
        for row in rows {
            providers.push(row?);
        }
        Ok(providers)
    }
}

fn row_to_model(row: &rusqlite::Row<'_>) -> rusqlite::Result<ModelMetadata> {
    let id: String = row.get(0)?;
    let provider: String = row.get(1)?;
    let context_limit: Option<i64> = row.get(2)?;
    let output_limit: Option<i64> = row.get(3)?;
    let reasoning: bool = row.get::<_, i64>(4)? != 0;
    let tool_call: bool = row.get::<_, i64>(5)? != 0;
    let structured_output: bool = row.get::<_, i64>(6)? != 0;
    let attachment: bool = row.get::<_, i64>(7)? != 0;
    let release_date: Option<String> = row.get(8)?;
    let source_str: String = row.get(9)?;
    let input_per_1m: Option<f64> = row.get(10)?;
    let output_per_1m: Option<f64> = row.get(11)?;
    let cache_read_per_1m: Option<f64> = row.get(12)?;

    let pricing =
        if input_per_1m.is_some() || output_per_1m.is_some() || cache_read_per_1m.is_some() {
            Some(ModelPricing {
                input_per_1m,
                output_per_1m,
                cache_read_per_1m,
            })
        } else {
            None
        };

    Ok(ModelMetadata {
        id,
        provider,
        context_limit: context_limit.map(|v| v as u32),
        output_limit: output_limit.map(|v| v as u32),
        pricing,
        capabilities: ModelCapabilities {
            tools: tool_call,
            structured_output,
            reasoning,
            image_input: attachment,
        },
        release_date,
        source: str_to_model_source(&source_str),
    })
}

fn row_to_provider(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProviderInfo> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let api_base_url: Option<String> = row.get(2)?;
    let env_var: Option<String> = row.get(3)?;
    let doc_url: Option<String> = row.get(4)?;
    let source_str: String = row.get(5)?;

    Ok(ProviderInfo {
        id,
        name,
        api_base_url,
        env_var,
        doc_url,
        source: str_to_provider_source(&source_str),
    })
}

fn model_source_to_str(source: &ModelSource) -> String {
    match source {
        ModelSource::Builtin => "builtin".to_string(),
        ModelSource::Manual => "manual".to_string(),
        ModelSource::ModelsDev { refreshed_at } => format!("models_dev:{refreshed_at}"),
    }
}

fn str_to_model_source(s: &str) -> ModelSource {
    if let Some(ts) = s.strip_prefix("models_dev:") {
        ModelSource::ModelsDev {
            refreshed_at: ts.to_string(),
        }
    } else if s == "manual" {
        ModelSource::Manual
    } else {
        ModelSource::Builtin
    }
}

fn source_to_str(source: &ProviderSource) -> String {
    match source {
        ProviderSource::Builtin => "builtin".to_string(),
        ProviderSource::ModelsDev { refreshed_at } => format!("models_dev:{refreshed_at}"),
    }
}

fn str_to_provider_source(s: &str) -> ProviderSource {
    if let Some(ts) = s.strip_prefix("models_dev:") {
        ProviderSource::ModelsDev {
            refreshed_at: ts.to_string(),
        }
    } else {
        ProviderSource::Builtin
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn sample_model(id: &str, provider: &str) -> ModelMetadata {
        ModelMetadata {
            id: id.to_string(),
            provider: provider.to_string(),
            context_limit: Some(200_000),
            output_limit: Some(8_192),
            pricing: Some(ModelPricing {
                input_per_1m: Some(3.0),
                output_per_1m: Some(15.0),
                cache_read_per_1m: Some(0.3),
            }),
            capabilities: ModelCapabilities {
                tools: true,
                structured_output: false,
                reasoning: true,
                image_input: true,
            },
            release_date: Some("2025-01-01".to_string()),
            source: ModelSource::Builtin,
        }
    }

    fn sample_provider(id: &str) -> ProviderInfo {
        ProviderInfo {
            id: id.to_string(),
            name: id.to_string(),
            api_base_url: Some(format!("https://api.{id}.com")),
            env_var: Some(format!("{}_API_KEY", id.to_uppercase())),
            doc_url: None,
            source: ProviderSource::Builtin,
        }
    }

    fn seed_models(catalog: &ModelCatalog, models: &[ModelMetadata], refreshed_at: &str) {
        catalog
            .seed(&[], models, refreshed_at)
            .expect("seed should succeed");
    }

    #[test]
    fn test_open_memory_creates_schema() {
        let catalog = ModelCatalog::open_memory().expect("open in-memory catalog");
        assert_eq!(catalog.schema_version().unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn test_open_creates_file() {
        let dir = tempfile::tempdir().expect("create temp dir");
        let db_path = dir.path().join("catalog.db");
        let catalog = ModelCatalog::open(&db_path).expect("open catalog");
        assert_eq!(catalog.schema_version().unwrap(), SCHEMA_VERSION);
        assert!(db_path.exists(), "database file should exist");
    }

    #[test]
    fn test_migrate_is_idempotent() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        catalog.migrate().expect("second migration should succeed");
        assert_eq!(catalog.schema_version().unwrap(), SCHEMA_VERSION);
    }

    #[test]
    fn test_seed_and_query_all() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        let models = vec![
            sample_model("claude-sonnet-4-5", "anthropic"),
            sample_model("gpt-4o", "openai"),
        ];
        seed_models(&catalog, &models, "2025-07-03T00:00:00Z");

        assert_eq!(catalog.model_count().unwrap(), 2);
        assert_eq!(catalog.provider_count().unwrap(), 2);

        let all = catalog.all_models().expect("all_models");
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].provider, "anthropic");
        assert_eq!(all[1].provider, "openai");
    }

    #[test]
    fn test_seed_sets_last_refreshed() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        seed_models(
            &catalog,
            &[sample_model("m1", "p1")],
            "2025-07-03T12:00:00Z",
        );
        assert_eq!(
            catalog.last_refreshed().unwrap(),
            Some("2025-07-03T12:00:00Z".to_string())
        );
    }

    #[test]
    fn test_seed_replaces_existing_data() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        seed_models(
            &catalog,
            &[sample_model("m1", "p1"), sample_model("m2", "p1")],
            "t1",
        );
        assert_eq!(catalog.model_count().unwrap(), 2);

        seed_models(&catalog, &[sample_model("m3", "p2")], "t2");
        assert_eq!(catalog.model_count().unwrap(), 1);
        assert_eq!(catalog.provider_count().unwrap(), 1);
    }

    #[test]
    fn test_find_model_found() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        seed_models(
            &catalog,
            &[sample_model("claude-sonnet-4-5", "anthropic")],
            "t",
        );

        let found = catalog
            .find_model("anthropic", "claude-sonnet-4-5")
            .expect("find_model");
        assert!(found.is_some());
        let m = found.unwrap();
        assert_eq!(m.context_limit, Some(200_000));
        assert_eq!(m.output_limit, Some(8_192));
        assert!(m.capabilities.tools);
        assert!(m.capabilities.reasoning);
        let p = m.pricing.as_ref().unwrap();
        assert_eq!(p.input_per_1m, Some(3.0));
    }

    #[test]
    fn test_find_model_missing_returns_none() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        seed_models(&catalog, &[sample_model("m1", "p1")], "t");

        assert!(catalog.find_model("p1", "nonexistent").unwrap().is_none());
        assert!(catalog.find_model("unknown", "m1").unwrap().is_none());
    }

    #[test]
    fn test_models_by_provider() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        seed_models(
            &catalog,
            &[
                sample_model("m1", "anthropic"),
                sample_model("m2", "anthropic"),
                sample_model("m3", "openai"),
            ],
            "t",
        );

        assert_eq!(catalog.models_by_provider("anthropic").unwrap().len(), 2);
        assert_eq!(catalog.models_by_provider("openai").unwrap().len(), 1);
    }

    #[test]
    fn test_model_count_for_provider() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        seed_models(
            &catalog,
            &[
                sample_model("m1", "p1"),
                sample_model("m2", "p1"),
                sample_model("m3", "p2"),
            ],
            "t",
        );
        assert_eq!(catalog.model_count_for_provider("p1").unwrap(), 2);
        assert_eq!(catalog.model_count_for_provider("p2").unwrap(), 1);
        assert_eq!(catalog.model_count_for_provider("missing").unwrap(), 0);
    }

    #[test]
    fn test_upsert_provider_and_query() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        catalog
            .upsert_provider(&sample_provider("anthropic"))
            .expect("upsert_provider");

        let providers = catalog.all_providers().expect("all_providers");
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].id, "anthropic");
        assert_eq!(
            providers[0].api_base_url.as_deref(),
            Some("https://api.anthropic.com")
        );
    }

    #[test]
    fn test_upsert_provider_replaces() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        catalog
            .upsert_provider(&sample_provider("anthropic"))
            .expect("upsert");

        let updated = ProviderInfo {
            name: "Anthropic Updated".to_string(),
            ..sample_provider("anthropic")
        };
        catalog.upsert_provider(&updated).expect("upsert");

        let providers = catalog.all_providers().expect("all_providers");
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].name, "Anthropic Updated");
    }

    #[test]
    fn test_upsert_model_with_pricing() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        catalog
            .upsert_provider(&sample_provider("test"))
            .expect("upsert_provider");
        catalog
            .upsert_model(&sample_model("m1", "test"))
            .expect("upsert_model");

        let found = catalog.find_model("test", "m1").unwrap().unwrap();
        assert!(found.pricing.is_some());
    }

    #[test]
    fn test_upsert_model_replaces_pricing() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        catalog
            .upsert_provider(&sample_provider("test"))
            .expect("upsert_provider");

        let mut model = sample_model("m1", "test");
        catalog.upsert_model(&model).expect("upsert_model");

        model.pricing = Some(ModelPricing {
            input_per_1m: Some(10.0),
            output_per_1m: Some(50.0),
            cache_read_per_1m: None,
        });
        catalog.upsert_model(&model).expect("upsert_model");

        let found = catalog.find_model("test", "m1").unwrap().unwrap();
        let p = found.pricing.as_ref().unwrap();
        assert_eq!(p.input_per_1m, Some(10.0));
        assert_eq!(p.output_per_1m, Some(50.0));
        assert_eq!(p.cache_read_per_1m, None);
    }

    #[test]
    fn test_search_models_by_id() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        seed_models(
            &catalog,
            &[
                sample_model("claude-sonnet-4-5", "anthropic"),
                sample_model("claude-haiku-4-5", "anthropic"),
                sample_model("gpt-4o", "openai"),
            ],
            "t",
        );

        assert_eq!(catalog.search_models("claude").unwrap().len(), 2);
        assert_eq!(catalog.search_models("gpt").unwrap().len(), 1);
    }

    #[test]
    fn test_search_models_by_provider() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        seed_models(
            &catalog,
            &[
                sample_model("m1", "anthropic"),
                sample_model("m2", "openai"),
            ],
            "t",
        );

        let results = catalog.search_models("anthropic").expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "m1");
    }

    #[test]
    fn test_search_providers() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        catalog
            .upsert_provider(&sample_provider("anthropic"))
            .expect("upsert");
        catalog
            .upsert_provider(&sample_provider("openai"))
            .expect("upsert");

        let results = catalog.search_providers("anth").expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "anthropic");
    }

    #[test]
    fn test_meta_set_get() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        assert!(catalog.get_meta("foo").unwrap().is_none());

        catalog.set_meta("foo", "bar").expect("set_meta");
        assert_eq!(catalog.get_meta("foo").unwrap(), Some("bar".to_string()));

        catalog.set_meta("foo", "baz").expect("set_meta");
        assert_eq!(catalog.get_meta("foo").unwrap(), Some("baz".to_string()));
    }

    #[test]
    fn test_empty_catalog_queries() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        assert!(catalog.all_models().unwrap().is_empty());
        assert!(catalog.all_providers().unwrap().is_empty());
        assert!(catalog.models_by_provider("none").unwrap().is_empty());
        assert!(catalog.find_model("none", "none").unwrap().is_none());
        assert!(catalog.search_models("test").unwrap().is_empty());
        assert_eq!(catalog.model_count().unwrap(), 0);
        assert_eq!(catalog.provider_count().unwrap(), 0);
    }

    #[test]
    fn test_source_roundtrip_models_dev() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        let model = ModelMetadata {
            id: "test".to_string(),
            provider: "test".to_string(),
            context_limit: None,
            output_limit: None,
            pricing: None,
            capabilities: ModelCapabilities::default(),
            release_date: None,
            source: ModelSource::ModelsDev {
                refreshed_at: "2025-07-03T00:00:00Z".to_string(),
            },
        };
        catalog
            .upsert_provider(&ProviderInfo {
                id: "test".to_string(),
                name: "test".to_string(),
                ..Default::default()
            })
            .expect("upsert_provider");
        catalog.upsert_model(&model).expect("upsert_model");

        let found = catalog.find_model("test", "test").unwrap().unwrap();
        assert_eq!(found.source, model.source);
    }

    #[test]
    fn test_corrupt_db_propagates_error() {
        let dir = tempfile::tempdir().expect("temp dir");
        let db_path = dir.path().join("corrupt.db");
        std::fs::write(&db_path, b"not a sqlite database").expect("write corrupt file");

        let result = ModelCatalog::open(&db_path);
        assert!(
            result.is_err(),
            "opening a corrupt DB must return an error, not panic"
        );
    }

    #[test]
    fn test_incompatible_schema_version_rejected() {
        let dir = tempfile::tempdir().expect("temp dir");
        let db_path = dir.path().join("future.db");

        {
            let catalog = ModelCatalog::open(&db_path).expect("create catalog");
            catalog
                .conn
                .execute("UPDATE schema_version SET version = 999", [])
                .expect("bump version");
        }

        let result = ModelCatalog::open(&db_path);
        let err = match result {
            Err(e) => e,
            Ok(_) => panic!("opening a future-schema DB must fail"),
        };
        assert!(
            matches!(
                err,
                CatalogError::IncompatibleSchema {
                    expected: 1,
                    found: 999
                }
            ),
            "expected IncompatibleSchema, got {err:?}"
        );
    }

    #[test]
    fn test_seed_empty_models() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        catalog.seed(&[], &[], "t").expect("seed empty");
        assert_eq!(catalog.model_count().unwrap(), 0);
        assert_eq!(catalog.provider_count().unwrap(), 0);
        assert_eq!(catalog.last_refreshed().unwrap(), Some("t".to_string()));
    }

    #[test]
    fn test_seed_with_provider_metadata() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        let providers = vec![
            ProviderInfo {
                id: "anthropic".to_string(),
                name: "Anthropic".to_string(),
                api_base_url: Some("https://api.anthropic.com".to_string()),
                env_var: Some("ANTHROPIC_API_KEY".to_string()),
                doc_url: Some("https://docs.anthropic.com".to_string()),
                source: ProviderSource::Builtin,
            },
            ProviderInfo {
                id: "openai".to_string(),
                name: "OpenAI".to_string(),
                api_base_url: Some("https://api.openai.com/v1".to_string()),
                env_var: Some("OPENAI_API_KEY".to_string()),
                doc_url: None,
                source: ProviderSource::Builtin,
            },
        ];
        let models = vec![
            sample_model("claude-sonnet-4-5", "anthropic"),
            sample_model("gpt-4o", "openai"),
        ];

        catalog.seed(&providers, &models, "t").expect("seed");

        let stored = catalog.all_providers().expect("all_providers");
        assert_eq!(stored.len(), 2);

        let anthropic = stored.iter().find(|p| p.id == "anthropic").unwrap();
        assert_eq!(anthropic.name, "Anthropic");
        assert_eq!(
            anthropic.api_base_url.as_deref(),
            Some("https://api.anthropic.com")
        );
        assert_eq!(anthropic.env_var.as_deref(), Some("ANTHROPIC_API_KEY"));
        assert_eq!(
            anthropic.doc_url.as_deref(),
            Some("https://docs.anthropic.com")
        );

        let openai = stored.iter().find(|p| p.id == "openai").unwrap();
        assert_eq!(openai.name, "OpenAI");
        assert_eq!(
            openai.api_base_url.as_deref(),
            Some("https://api.openai.com/v1")
        );
    }

    #[test]
    fn test_seed_auto_derives_missing_providers() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        catalog
            .seed(
                &[sample_provider("known")],
                &[
                    sample_model("m1", "known"),
                    sample_model("m2", "unknown-provider"),
                ],
                "t",
            )
            .expect("seed");

        let providers = catalog.all_providers().expect("all_providers");
        assert_eq!(providers.len(), 2);

        let known = providers.iter().find(|p| p.id == "known").unwrap();
        assert_eq!(known.api_base_url.as_deref(), Some("https://api.known.com"));

        let unknown = providers
            .iter()
            .find(|p| p.id == "unknown-provider")
            .unwrap();
        assert!(unknown.api_base_url.is_none());
    }

    #[test]
    fn test_model_without_pricing() {
        let catalog = ModelCatalog::open_memory().expect("open catalog");
        let model = ModelMetadata {
            id: "free-model".to_string(),
            provider: "test".to_string(),
            context_limit: Some(100_000),
            output_limit: None,
            pricing: None,
            capabilities: ModelCapabilities::default(),
            release_date: None,
            source: ModelSource::Builtin,
        };
        seed_models(&catalog, &[model], "t");

        let found = catalog.find_model("test", "free-model").unwrap().unwrap();
        assert!(found.pricing.is_none());
        assert_eq!(found.context_limit, Some(100_000));
    }
}
