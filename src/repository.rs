use std::path::{Path, PathBuf};

use crate::{Error, Result};

const CONFIG_NAME: &str = "petridish.yaml";

pub trait Repository {
    fn repo(&self) -> String;
    fn kind(&self) -> &'static str;
    fn determine_repo_dir(&self) -> &Path;
    fn cached(&self) -> bool {
        false
    }
    fn download(&self) -> Result<()>;
    fn validate(&self) -> Result<()> {
        let repo_dir = self.determine_repo_dir();
        let repo = self.repo();
        if !repo_dir.exists() {
            return Err(Error::Repo {
                name: repo,
                error: "repo dir not found".into(),
            });
        }

        let config_path = repo_dir.join(CONFIG_NAME);
        if !config_path.exists() {
            return Err(Error::Repo {
                name: repo,
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

    fn repo(&self) -> String {
        self.path.display().to_string()
    }

    fn determine_repo_dir(&self) -> &Path {
        &self.path
    }

    fn download(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod directory_tests {
    use std::fs;

    use super::*;

    use rstest::*;
    use tempdir::TempDir;

    #[fixture]
    fn repo() -> Directory {
        Directory {
            path: PathBuf::from("/a/b"),
        }
    }

    #[rstest]
    fn kind(repo: Directory) {
        assert_eq!(repo.kind(), "directory")
    }

    #[rstest]
    fn determine_repo_dir(repo: Directory) {
        assert_eq!(repo.determine_repo_dir(), &PathBuf::from("/a/b"))
    }

    #[rstest]
    fn validate_uncreated_repo_dir(repo: Directory) {
        assert!(matches!(
                repo.validate(),
                Err(Error::Repo { name, error }) if name == "/a/b" && error == "repo dir not found"))
    }

    #[rstest]
    fn validate_created_repo_dir_which_missing_config() {
        let repo_dir = TempDir::new("template").unwrap();
        let repo = Directory {
            path: PathBuf::from(repo_dir.path()),
        };

        let config_path = repo_dir.path().join(CONFIG_NAME);

        assert!(matches!(
                repo.validate(),
                Err(Error::Repo { name: _, error }) if error == format!("config '{}' not found", config_path.display())));
        repo_dir.close().unwrap();
    }

    #[rstest]
    fn validate_created_repo_dir_which_has_config() {
        let repo_dir = TempDir::new("template").unwrap();
        let repo = Directory {
            path: PathBuf::from(repo_dir.path()),
        };

        let config_path = repo_dir.path().join(CONFIG_NAME);
        fs::write(&config_path, "").unwrap();

        assert!(repo.validate().is_ok());
        repo_dir.close().unwrap();
    }
}
