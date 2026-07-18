use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentModelEntry {
    pub provider: String,
    pub model_id: String,
    pub variant: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecentModelList {
    pub entries: Vec<RecentModelEntry>,
}

impl RecentModelList {
    pub fn record(&mut self, entry: RecentModelEntry) {
        self.entries.retain(|e| {
            e.provider != entry.provider
                || e.model_id != entry.model_id
                || e.variant != entry.variant
        });
        self.entries.insert(0, entry);
        if self.entries.len() > 5 {
            self.entries.truncate(5);
        }
    }
}

fn recent_models_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".talos")
        .join("recent_models.json")
}

pub fn load_recent_models(test_path: Option<&std::path::Path>) -> RecentModelList {
    let path = test_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(recent_models_path);
    if let Ok(content) = fs::read_to_string(&path)
        && let Ok(list) = serde_json::from_str::<RecentModelList>(&content)
    {
        return list;
    }
    RecentModelList::default()
}

pub fn save_recent_models(
    list: &RecentModelList,
    test_path: Option<&std::path::Path>,
) -> Result<(), anyhow::Error> {
    let path = test_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(recent_models_path);
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        let _ = fs::create_dir_all(parent);
    }

    let content = serde_json::to_string_pretty(list)?;
    let temp_path = path.with_extension("json.tmp");

    if let Err(e) = fs::write(&temp_path, content) {
        tracing::warn!("Failed to write recent models temp file: {}", e);
        return Err(anyhow::anyhow!(
            "Failed to write recent models temp file: {}",
            e
        ));
    }

    if let Err(e) = fs::rename(&temp_path, &path) {
        tracing::warn!("Failed to commit recent models file: {}", e);
        let _ = fs::remove_file(&temp_path);
        return Err(anyhow::anyhow!(
            "Failed to commit recent models file: {}",
            e
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_model_list_record() {
        let mut list = RecentModelList::default();

        list.record(RecentModelEntry {
            provider: "p1".into(),
            model_id: "m1".into(),
            variant: None,
        });
        list.record(RecentModelEntry {
            provider: "p2".into(),
            model_id: "m2".into(),
            variant: Some("v1".into()),
        });

        assert_eq!(list.entries.len(), 2);
        assert_eq!(list.entries[0].provider, "p2");
        assert_eq!(list.entries[1].provider, "p1");

        // Dedup: inserting same brings to front
        list.record(RecentModelEntry {
            provider: "p1".into(),
            model_id: "m1".into(),
            variant: None,
        });
        assert_eq!(list.entries.len(), 2);
        assert_eq!(list.entries[0].provider, "p1");

        // Eviction > 5
        for i in 0..10 {
            list.record(RecentModelEntry {
                provider: format!("p{i}"),
                model_id: "m".into(),
                variant: None,
            });
        }
        assert_eq!(list.entries.len(), 5);
        assert_eq!(list.entries[0].provider, "p9");
    }

    #[test]
    fn test_load_save_recent_models() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("recent.json");

        let mut list = load_recent_models(Some(&path));
        assert!(list.entries.is_empty());

        list.record(RecentModelEntry {
            provider: "p1".into(),
            model_id: "m1".into(),
            variant: None,
        });
        assert!(save_recent_models(&list, Some(&path)).is_ok());

        let loaded = load_recent_models(Some(&path));
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].provider, "p1");
    }
}
