use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::syntax::SyntaxLanguage;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedFile {
    pub path: PathBuf,
    pub text: String,
    pub language: SyntaxLanguage,
    pub stamp: FileStamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileStamp {
    pub length: u64,
    pub modified: Option<SystemTime>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedFile {
    pub path: PathBuf,
    pub stamp: FileStamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileError {
    Read { path: PathBuf, message: String },
    InvalidUtf8 { path: PathBuf },
    Modified { path: PathBuf },
    Write { path: PathBuf, message: String },
}

pub fn load(path: impl AsRef<Path>) -> Result<LoadedFile, FileError> {
    let path = path.as_ref().to_path_buf();
    let bytes = fs::read(&path).map_err(|error| FileError::Read {
        path: path.clone(),
        message: error.to_string(),
    })?;
    let text =
        String::from_utf8(bytes).map_err(|_| FileError::InvalidUtf8 { path: path.clone() })?;
    let language = SyntaxLanguage::from_path(&path);
    Ok(LoadedFile {
        stamp: file_stamp(&path).map_err(|error| FileError::Read {
            path: path.clone(),
            message: error.to_string(),
        })?,
        path,
        text,
        language,
    })
}

pub fn save_if_unchanged(
    path: impl AsRef<Path>,
    text: &str,
    expected: Option<&FileStamp>,
) -> Result<SavedFile, FileError> {
    let path = path.as_ref().to_path_buf();
    if let Some(expected) = expected {
        let current = file_stamp(&path).map_err(|error| FileError::Write {
            path: path.clone(),
            message: error.to_string(),
        })?;
        if &current != expected {
            return Err(FileError::Modified { path });
        }
    }
    let temporary_path = path.with_extension(format!(
        "{}functors-tmp-{}",
        path.extension()
            .and_then(|extension| extension.to_str())
            .map_or_else(String::new, |extension| format!("{extension}.")),
        std::process::id()
    ));

    fs::write(&temporary_path, text).map_err(|error| FileError::Write {
        path: path.clone(),
        message: error.to_string(),
    })?;

    #[cfg(windows)]
    if path.exists() {
        fs::remove_file(&path).map_err(|error| FileError::Write {
            path: path.clone(),
            message: error.to_string(),
        })?;
    }

    if let Err(error) = fs::rename(&temporary_path, &path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(FileError::Write {
            path,
            message: error.to_string(),
        });
    }

    let stamp = file_stamp(&path).map_err(|error| FileError::Write {
        path: path.clone(),
        message: error.to_string(),
    })?;
    Ok(SavedFile { path, stamp })
}

fn file_stamp(path: &Path) -> std::io::Result<FileStamp> {
    let metadata = fs::metadata(path)?;
    Ok(FileStamp {
        length: metadata.len(),
        modified: metadata.modified().ok(),
    })
}

#[cfg(test)]
mod tests {
    use super::{FileError, load, save_if_unchanged};
    use crate::syntax::SyntaxLanguage;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn loads_utf8_text_and_selects_language_from_extension() {
        let path = test_path("source.rs");
        fs::write(&path, "fn main() {}\n").unwrap();

        let loaded = load(&path).unwrap();

        assert_eq!(loaded.path, path);
        assert_eq!(loaded.text, "fn main() {}\n");
        assert_eq!(loaded.language, SyntaxLanguage::Rust);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn saves_by_replacing_the_target_file() {
        let path = test_path("notes.md");
        fs::write(&path, "# Original\n").unwrap();
        save_if_unchanged(&path, "# Updated\n", None).unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "# Updated\n");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn reports_invalid_utf8_without_returning_partial_text() {
        let path = test_path("binary.txt");
        fs::write(&path, [0xff, 0xfe]).unwrap();

        assert_eq!(
            load(&path),
            Err(FileError::InvalidUtf8 { path: path.clone() })
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_save_when_file_changed_after_load() {
        let path = test_path("changed.rs");
        fs::write(&path, "fn main() {}\n").unwrap();
        let loaded = load(&path).unwrap();
        fs::write(&path, "fn main() { println!(\"external\"); }\n").unwrap();

        assert_eq!(
            save_if_unchanged(&path, &loaded.text, Some(&loaded.stamp)),
            Err(FileError::Modified { path: path.clone() })
        );
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            "fn main() { println!(\"external\"); }\n"
        );
        fs::remove_file(path).unwrap();
    }

    fn test_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("functors-file-{}-{name}", std::process::id()))
    }
}
