use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use crate::{Error, Result};

const CONFIG_NAME: &str = "petridish.yaml";

pub trait Repository: Display {
    fn kind(&self) -> &'static str;
    fn determine_repo_dir(&self) -> &Path;
    fn cached(&self) -> bool {
        false
    }
    fn sync(&self) -> Result<()>;
    fn validate(&self) -> Result<()> {
        let repo_dir = self.determine_repo_dir();
        let name = self.to_string();
        if !repo_dir.exists() {
            return Err(Error::Repo {
                name,
                error: "repo dir not found".into(),
            });
        }

        let config_path = repo_dir.join(CONFIG_NAME);
        if !config_path.exists() {
            return Err(Error::Repo {
                name,
                error: format!("config '{}' not found", config_path.display()),
            });
        }

        Ok(())
    }
}

struct Directory {
    path: PathBuf,
}

impl Repository for Directory {
    fn kind(&self) -> &'static str {
        "directory"
    }

    fn determine_repo_dir(&self) -> &Path {
        &self.path
    }

    fn sync(&self) -> Result<()> {
        Ok(())
    }
}

impl Display for Directory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

#[cfg(test)]
mod directory_tests {
    use std::fs;

    use rstest::*;
    use tempdir::TempDir;

    use super::*;

    #[fixture]
    fn repo() -> Directory {
        Directory {
            path: PathBuf::from("/a/b"),
        }
    }

    #[rstest]
    fn check_kind(repo: Directory) {
        assert_eq!(repo.kind(), "directory")
    }

    #[rstest]
    fn determine_repo_dir(repo: Directory) {
        assert_eq!(repo.determine_repo_dir(), &PathBuf::from("/a/b"))
    }

    #[rstest]
    fn validate_repo_dir() {
        let repo_dir = TempDir::new("template").unwrap();
        let repo = Directory {
            path: PathBuf::from(repo_dir.path()),
        };

        let config_path = repo_dir.path().join(CONFIG_NAME);
        fs::write(&config_path, "").unwrap();

        assert!(repo.validate().is_ok());
    }

    #[rstest]
    fn validate_repo_dir_while_missing_config() {
        let repo_dir = TempDir::new("template").unwrap();
        let repo = Directory {
            path: PathBuf::from(repo_dir.path()),
        };

        let config_path = repo_dir.path().join(CONFIG_NAME);

        assert!(matches!(
                repo.validate(),
                Err(Error::Repo { name: _, error }) if error == format!("config '{}' not found", config_path.display())));
    }

    #[rstest]
    fn validate_nonexistent_repo_dir(repo: Directory) {
        assert!(matches!(
                repo.validate(),
                Err(Error::Repo { name, error }) if name == "/a/b" && error == "repo dir not found"))
    }
}
