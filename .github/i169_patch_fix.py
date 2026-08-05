from pathlib import Path

path = Path(".github/i169_reconciliation_windows_patch.py")
text = path.read_text()
old = '''def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"expected one match in {path}, found {count}: {old[:120]!r}")
    path.write_text(text.replace(old, new, 1))
'''
new = '''def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    count = text.count(old)
    expected = 2 if "HTTP/1.1 302 Found" in old else 1
    if count != expected:
        raise RuntimeError(
            f"expected {expected} match(es) in {path}, found {count}: {old[:120]!r}"
        )
    path.write_text(text.replace(old, new, expected))
'''
if text.count(old) != 1:
    raise RuntimeError("patch helper boundary changed")
text = text.replace(old, new, 1)

connection_patch_start = text.index(
    "replace_once(\n    pending,\n    '''    fn connection(&self) -> Result<Connection, PendingSubmissionError> {\n"
)
connection_patch_end = text.index("\n\nartifacts = ROOT", connection_patch_start)
identity_scoped_marker_patch = r"""replace_once(
    pending,
    '''    pub fn initialize_runtime_identity(
        &self,
        identity: SessionRuntimeIdentity,
    ) -> Result<SessionRuntimeState, PendingSubmissionError> {
        let _guard = self.guard()?;
''',
    '''    pub fn initialize_runtime_identity(
        &self,
        identity: SessionRuntimeIdentity,
    ) -> Result<SessionRuntimeState, PendingSubmissionError> {
        self.ensure_transcript_owner_marker()?;
        let _guard = self.guard()?;
''',
)
replace_once(
    pending,
    '''    fn connection(&self) -> Result<Connection, PendingSubmissionError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(self.path.as_ref())?;
''',
    '''    fn ensure_transcript_owner_marker(&self) -> Result<(), PendingSubmissionError> {
        if self.session_file.exists() {
            return Ok(());
        }
        if let Some(parent) = self.session_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(self.session_file.as_ref())
        {
            Ok(_) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    fn connection(&self) -> Result<Connection, PendingSubmissionError> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let connection = Connection::open(self.path.as_ref())?;
''',
)
"""
text = (
    text[:connection_patch_start]
    + identity_scoped_marker_patch
    + text[connection_patch_end:]
)
path.write_text(text)
