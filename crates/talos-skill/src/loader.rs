use crate::parser::{split_frontmatter, validate_frontmatter};
use crate::{
    Result, Skill, SkillError, SkillFrontmatter, SkillIndex, SkillSource, estimate_tokens,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const DEFAULT_MAX_SKILL_DISCOVERY_DEPTH: usize = 32;
const DEFAULT_MAX_SKILL_DISCOVERY_ENTRIES: usize = 10_000;

pub enum ExternalTargetPolicy {
    DenyOutsideSearchRoot,
    AllowAnyReadable,
}

pub struct SkillDiscoveryPolicy {
    pub follow_directory_links: bool,
    pub external_target_policy: ExternalTargetPolicy,
    pub max_depth: usize,
    pub max_entries: usize,
}

impl Default for SkillDiscoveryPolicy {
    fn default() -> Self {
        Self {
            follow_directory_links: false,
            external_target_policy: ExternalTargetPolicy::DenyOutsideSearchRoot,
            max_depth: DEFAULT_MAX_SKILL_DISCOVERY_DEPTH,
            max_entries: DEFAULT_MAX_SKILL_DISCOVERY_ENTRIES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillDiscoveryWarningKind {
    BrokenLink,
    LinkLoop,
    PermissionDenied,
    ExternalTargetDenied,
    CanonicalizeFailed,
    DepthLimitReached,
    EntryBudgetReached,
    InvalidSkill,
    Io,
}

#[derive(Debug, Clone)]
pub struct SkillDiscoveryWarning {
    pub kind: SkillDiscoveryWarningKind,
    pub path: PathBuf,
    pub message: String,
}

pub struct SkillLoader {
    pub skills: Vec<Skill>,
    pub search_paths: Vec<PathBuf>,
    pub discover_shared: bool,
    pub workspace_root: Option<PathBuf>,
    pub discovery_policy: SkillDiscoveryPolicy,
    pub discovery_warnings: Vec<SkillDiscoveryWarning>,
}

impl SkillLoader {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().ok();
        Self {
            skills: Vec::new(),
            search_paths: default_search_paths(cwd.as_deref(), false),
            discover_shared: false,
            workspace_root: cwd.map(|p| p.to_path_buf()),
            discovery_policy: SkillDiscoveryPolicy::default(),
            discovery_warnings: Vec::new(),
        }
    }

    pub fn for_workspace(workspace_root: impl AsRef<Path>) -> Self {
        Self::for_workspace_with_options(workspace_root.as_ref(), false)
    }

    pub fn for_workspace_with_options(
        workspace_root: impl AsRef<Path>,
        discover_shared: bool,
    ) -> Self {
        let root = workspace_root.as_ref();
        Self {
            skills: Vec::new(),
            search_paths: default_search_paths(Some(root), discover_shared),
            discover_shared,
            workspace_root: Some(root.to_path_buf()),
            discovery_policy: SkillDiscoveryPolicy::default(),
            discovery_warnings: Vec::new(),
        }
    }

    pub fn for_workspace_with_discovery_policy(
        workspace_root: impl AsRef<Path>,
        discover_shared: bool,
        policy: SkillDiscoveryPolicy,
    ) -> Self {
        let root = workspace_root.as_ref();
        Self {
            skills: Vec::new(),
            search_paths: default_search_paths(Some(root), discover_shared),
            discover_shared,
            workspace_root: Some(root.to_path_buf()),
            discovery_policy: policy,
            discovery_warnings: Vec::new(),
        }
    }

    pub fn discovery_warnings(&self) -> &[SkillDiscoveryWarning] {
        &self.discovery_warnings
    }

    pub fn discover(&mut self) -> Result<&Vec<Skill>> {
        self.skills.clear();
        self.discovery_warnings.clear();

        let mut seen_skill_names: HashSet<String> = HashSet::new();
        let mut seen_canonical_dirs: HashSet<PathBuf> = HashSet::new();
        let mut seen_canonical_files: HashSet<PathBuf> = HashSet::new();
        let mut entry_count: usize = 0;

        for search_root in &self.search_paths {
            if !search_root.is_dir() {
                continue;
            }

            let root_canonical = match search_root.canonicalize() {
                Ok(c) => c,
                Err(_) => search_root.clone(),
            };
            if !seen_canonical_dirs.insert(root_canonical.clone()) {
                continue;
            }

            let source = self.classify_source(search_root);
            let follow = self.discovery_policy.follow_directory_links;
            let max_depth = self.discovery_policy.max_depth;
            let max_entries = self.discovery_policy.max_entries;

            let walker = WalkDir::new(search_root)
                .follow_links(follow)
                .max_depth(max_depth)
                .into_iter();

            for result in walker {
                entry_count += 1;
                if entry_count > max_entries {
                    self.discovery_warnings.push(SkillDiscoveryWarning {
                        kind: SkillDiscoveryWarningKind::EntryBudgetReached,
                        path: search_root.clone(),
                        message: format!(
                            "entry budget {max_entries} reached; remaining entries skipped"
                        ),
                    });
                    break;
                }

                let entry = match result {
                    Ok(e) => e,
                    Err(e) => {
                        let kind = if e.loop_ancestor().is_some() {
                            SkillDiscoveryWarningKind::LinkLoop
                        } else if e.depth() > max_depth {
                            SkillDiscoveryWarningKind::DepthLimitReached
                        } else if e
                            .path()
                            .map(|p| {
                                std::fs::symlink_metadata(p)
                                    .map(|m| m.file_type().is_symlink() && !p.exists())
                                    .unwrap_or(false)
                            })
                            .unwrap_or(false)
                        {
                            SkillDiscoveryWarningKind::BrokenLink
                        } else {
                            SkillDiscoveryWarningKind::Io
                        };
                        let path = e.path().map(|p| p.to_path_buf()).unwrap_or_default();
                        self.discovery_warnings.push(SkillDiscoveryWarning {
                            kind,
                            path,
                            message: e.to_string(),
                        });
                        continue;
                    }
                };

                let entry_path = entry.path();
                if entry_path.file_name() != Some(std::ffi::OsStr::new("SKILL.md")) {
                    continue;
                }

                if follow {
                    if let Some(parent) = entry_path.parent() {
                        match parent.canonicalize() {
                            Ok(canon_dir) => {
                                if !self.is_target_allowed(&canon_dir, &root_canonical) {
                                    self.discovery_warnings.push(SkillDiscoveryWarning {
                                        kind: SkillDiscoveryWarningKind::ExternalTargetDenied,
                                        path: entry_path.to_path_buf(),
                                        message: format!(
                                            "target {canon_dir:?} is outside search root {root_canonical:?}"
                                        ),
                                    });
                                    continue;
                                }
                                if !seen_canonical_dirs.insert(canon_dir) {
                                    continue;
                                }
                            }
                            Err(e) => {
                                self.discovery_warnings.push(SkillDiscoveryWarning {
                                    kind: SkillDiscoveryWarningKind::CanonicalizeFailed,
                                    path: parent.to_path_buf(),
                                    message: e.to_string(),
                                });
                                continue;
                            }
                        }
                    }

                    match entry_path.canonicalize() {
                        Ok(canon_file) => {
                            if !seen_canonical_files.insert(canon_file) {
                                continue;
                            }
                        }
                        Err(e) => {
                            self.discovery_warnings.push(SkillDiscoveryWarning {
                                kind: SkillDiscoveryWarningKind::CanonicalizeFailed,
                                path: entry_path.to_path_buf(),
                                message: e.to_string(),
                            });
                            continue;
                        }
                    }
                }

                match Self::parse(entry_path) {
                    Ok(mut skill) => {
                        if seen_skill_names.insert(skill.name.clone()) {
                            skill.source = source;
                            self.skills.push(skill);
                        }
                    }
                    Err(e) => {
                        self.discovery_warnings.push(SkillDiscoveryWarning {
                            kind: SkillDiscoveryWarningKind::InvalidSkill,
                            path: entry_path.to_path_buf(),
                            message: e.to_string(),
                        });
                    }
                }
            }
        }

        Ok(&self.skills)
    }

    fn is_target_allowed(&self, target: &Path, root: &Path) -> bool {
        match self.discovery_policy.external_target_policy {
            ExternalTargetPolicy::AllowAnyReadable => true,
            ExternalTargetPolicy::DenyOutsideSearchRoot => target.starts_with(root),
        }
    }

    fn classify_source(&self, path: &Path) -> SkillSource {
        if let Some(ref home) = home_dir() {
            let agents_skills = home.join(".agents").join("skills");
            if path.starts_with(&agents_skills) {
                return SkillSource::Shared;
            }
            let talos_skills = home.join(".talos").join("skills");
            if path.starts_with(&talos_skills) {
                return SkillSource::UserGlobal;
            }
        }

        if let Some(ref root) = self.workspace_root {
            let project_skills = root.join(".talos").join("skills");
            if path.starts_with(&project_skills) {
                return SkillSource::Project;
            }
        }

        SkillSource::Parent
    }

    pub fn parse(path: &Path) -> Result<Skill> {
        if !path.exists() {
            return Err(SkillError::FileNotFound(path.to_path_buf()));
        }

        let content = std::fs::read_to_string(path)?;
        let (frontmatter, body) = split_frontmatter(&content)?;
        let fm: SkillFrontmatter = serde_yaml::from_str(frontmatter)?;

        validate_frontmatter(&fm)?;

        Ok(Skill {
            name: fm.name,
            description: fm.description,
            triggers: fm.triggers,
            body: body.trim().to_string(),
            source_path: path.to_path_buf(),
            source: SkillSource::default(),
        })
    }

    pub fn get_index(&self) -> Vec<SkillIndex> {
        self.skills
            .iter()
            .map(|s| {
                let level0_text = format!("{}: {}", s.name, s.description);
                SkillIndex {
                    name: s.name.clone(),
                    description: s.description.clone(),
                    triggers: s.triggers.clone(),
                    estimated_tokens: estimate_tokens(&level0_text),
                    source: s.source,
                }
            })
            .collect()
    }
}

impl Default for SkillLoader {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn home_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok().map(PathBuf::from)
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

fn default_search_paths(workspace_root: Option<&Path>, discover_shared: bool) -> Vec<PathBuf> {
    let mut search_paths = Vec::new();

    if let Some(root) = workspace_root {
        push_if_dir(&mut search_paths, root.join(".talos/skills"));
    }

    if let Some(home) = home_dir() {
        push_if_dir(&mut search_paths, home.join(".talos/skills"));
    }

    if let Some(root) = workspace_root {
        let mut current = root;
        while let Some(parent) = current.parent() {
            let git_dir = parent.join(".git");
            push_if_dir(&mut search_paths, parent.join(".talos/skills"));
            current = parent;
            if git_dir.is_dir() {
                break;
            }
        }
    }

    if discover_shared && let Some(home) = home_dir() {
        push_if_dir(&mut search_paths, home.join(".agents/skills"));
    }

    search_paths
}

fn push_if_dir(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.is_dir() && !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}
