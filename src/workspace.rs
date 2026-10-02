use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workspace {
    pub root: PathBuf,
    pub entries: Vec<WorkspaceEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceEntry {
    pub name: String,
    pub path: PathBuf,
    pub kind: WorkspaceEntryKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceEntryKind {
    File,
    Directory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceError {
    InvalidRoot(PathBuf),
    ReadFailed { path: PathBuf, message: String },
}

pub fn discover(root: impl AsRef<Path>) -> Result<Workspace, WorkspaceError> {
    let root = root.as_ref().to_path_buf();
    if !root.is_dir() {
        return Err(WorkspaceError::InvalidRoot(root));
    }

    let mut entries = fs::read_dir(&root)
        .map_err(|error| WorkspaceError::ReadFailed {
            path: root.clone(),
            message: error.to_string(),
        })?
        .map(|entry| {
            let entry = entry.map_err(|error| WorkspaceError::ReadFailed {
                path: root.clone(),
                message: error.to_string(),
            })?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| WorkspaceError::ReadFailed {
                    path: path.clone(),
                    message: error.to_string(),
                })?;
            Ok(WorkspaceEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                path,
                kind: if file_type.is_dir() {
                    WorkspaceEntryKind::Directory
                } else {
                    WorkspaceEntryKind::File
                },
            })
        })
        .collect::<Result<Vec<_>, WorkspaceError>>()?;

    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(Workspace { root, entries })
}

#[cfg(test)]
mod tests {
    use super::{Workspace, WorkspaceEntry, WorkspaceEntryKind, WorkspaceError, discover};
    use std::fs;

    #[test]
    fn deterministic_fixture_has_stable_domain_data() {
        let fixture = Workspace {
            root: "fixture".into(),
            entries: vec![
                WorkspaceEntry {
                    name: "Cargo.toml".into(),
                    path: "fixture/Cargo.toml".into(),
                    kind: WorkspaceEntryKind::File,
                },
                WorkspaceEntry {
                    name: "src".into(),
                    path: "fixture/src".into(),
                    kind: WorkspaceEntryKind::Directory,
                },
            ],
        };

        assert_eq!(fixture.root, std::path::PathBuf::from("fixture"));
        assert_eq!(fixture.entries[0].kind, WorkspaceEntryKind::File);
        assert_eq!(fixture.entries[1].kind, WorkspaceEntryKind::Directory);
    }

    #[test]
    fn discovers_sorted_files_and_directories() {
        let root = unique_test_directory("workspace");
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("README.md"), "# Functors").unwrap();
        fs::write(root.join("Cargo.toml"), "[package]").unwrap();

        let workspace = discover(&root).unwrap();

        assert_eq!(workspace.root, root);
        assert_eq!(
            workspace
                .entries
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Cargo.toml", "README.md", "src"]
        );
        assert_eq!(workspace.entries[2].kind, WorkspaceEntryKind::Directory);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_a_non_directory_root() {
        let root =
            std::env::temp_dir().join(format!("functors-workspace-file-{}", std::process::id()));
        fs::write(&root, "file").unwrap();

        assert_eq!(
            discover(&root),
            Err(WorkspaceError::InvalidRoot(root.clone()))
        );
        fs::remove_file(root).unwrap();
    }

    fn unique_test_directory(label: &str) -> std::path::PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("functors-{label}-{}-{suffix}", std::process::id()));
        fs::create_dir(&root).unwrap();
        root
    }
}
