//! Madara DB schema version detection.
//!
//! Upstream Madara persists the database schema version in a sidecar file named
//! `.db-version` (UTF-8 text, a single decimal `u32`), stored in the node
//! base-path directory (next to the `db/` RocksDB directory).
//!
//! Reference (upstream): `madara/crates/client/db/src/migration/mod.rs`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::DbReader;

/// Upstream Madara DB version file name.
pub const MADARA_DB_VERSION_FILE: &str = ".db-version";

/// Result of a best-effort DB version detection.
#[derive(Debug, Clone)]
pub struct MadaraDbVersionDetection {
    /// Parsed DB schema version, if detected.
    pub version: Option<u32>,
    /// Path of the version file used for detection, if any.
    pub source_path: Option<PathBuf>,
    /// Human-friendly error (e.g. file exists but cannot be parsed).
    pub error: Option<String>,
}

impl MadaraDbVersionDetection {
    fn none() -> Self {
        Self {
            version: None,
            source_path: None,
            error: None,
        }
    }
}

/// Detect the Madara DB schema version given the RocksDB directory path.
///
/// We check:
/// 1. `<db_path>/../.db-version` (canonical upstream location)
/// 2. `<db_path>/.db-version` (fallback, in case the caller passed base-path or a non-standard layout)
pub fn detect_madara_db_version_for_db_path(db_path: &Path) -> MadaraDbVersionDetection {
    let mut candidates: Vec<PathBuf> = Vec::with_capacity(2);
    if let Some(parent) = db_path.parent() {
        candidates.push(parent.join(MADARA_DB_VERSION_FILE));
    }
    candidates.push(db_path.join(MADARA_DB_VERSION_FILE));

    for path in candidates {
        if !path.exists() {
            continue;
        }

        if !path.is_file() {
            return MadaraDbVersionDetection {
                version: None,
                source_path: Some(path),
                error: Some("version file exists but is not a regular file".to_string()),
            };
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                return MadaraDbVersionDetection {
                    version: None,
                    source_path: Some(path),
                    error: Some(format!("failed to read version file: {e}")),
                };
            }
        };

        let trimmed = content.trim();
        match trimmed.parse::<u32>() {
            Ok(version) => {
                return MadaraDbVersionDetection {
                    version: Some(version),
                    source_path: Some(path),
                    error: None,
                };
            }
            Err(e) => {
                // Don't keep scanning other paths; if a version file exists but is invalid, surface it.
                return MadaraDbVersionDetection {
                    version: None,
                    source_path: Some(path),
                    error: Some(format!("invalid version content {trimmed:?}: {e}")),
                };
            }
        }
    }

    MadaraDbVersionDetection::none()
}

impl DbReader {
    /// Best-effort detection of the Madara DB schema version.
    pub fn detect_madara_db_version(&self) -> MadaraDbVersionDetection {
        detect_madara_db_version_for_db_path(self.path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn detects_from_parent_dir() {
        let base = TempDir::new().unwrap();
        let db_dir = base.path().join("db");
        fs::create_dir_all(&db_dir).unwrap();
        fs::write(base.path().join(MADARA_DB_VERSION_FILE), "9\n").unwrap();

        let det = detect_madara_db_version_for_db_path(&db_dir);
        assert_eq!(det.version, Some(9));
        assert_eq!(
            det.source_path,
            Some(base.path().join(MADARA_DB_VERSION_FILE))
        );
        assert!(det.error.is_none());
    }

    #[test]
    fn detects_from_db_dir_fallback() {
        let base = TempDir::new().unwrap();
        let db_dir = base.path().join("db");
        fs::create_dir_all(&db_dir).unwrap();
        fs::write(db_dir.join(MADARA_DB_VERSION_FILE), "8").unwrap();

        let det = detect_madara_db_version_for_db_path(&db_dir);
        assert_eq!(det.version, Some(8));
        assert_eq!(det.source_path, Some(db_dir.join(MADARA_DB_VERSION_FILE)));
        assert!(det.error.is_none());
    }

    #[test]
    fn prefers_parent_when_both_exist() {
        let base = TempDir::new().unwrap();
        let db_dir = base.path().join("db");
        fs::create_dir_all(&db_dir).unwrap();
        fs::write(base.path().join(MADARA_DB_VERSION_FILE), "9").unwrap();
        fs::write(db_dir.join(MADARA_DB_VERSION_FILE), "8").unwrap();

        let det = detect_madara_db_version_for_db_path(&db_dir);
        assert_eq!(det.version, Some(9));
        assert_eq!(
            det.source_path,
            Some(base.path().join(MADARA_DB_VERSION_FILE))
        );
        assert!(det.error.is_none());
    }

    #[test]
    fn surfaces_invalid_content() {
        let base = TempDir::new().unwrap();
        let db_dir = base.path().join("db");
        fs::create_dir_all(&db_dir).unwrap();
        fs::write(base.path().join(MADARA_DB_VERSION_FILE), "invalid").unwrap();

        let det = detect_madara_db_version_for_db_path(&db_dir);
        assert_eq!(det.version, None);
        assert_eq!(
            det.source_path,
            Some(base.path().join(MADARA_DB_VERSION_FILE))
        );
        assert!(det
            .error
            .as_deref()
            .unwrap_or_default()
            .contains("invalid version content"));
    }
}
