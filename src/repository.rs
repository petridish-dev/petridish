use std::{fmt::Display, fs, path::PathBuf, process::Command};

use dirs::cache_dir;
use md5;

use crate::{Error, Result};

const CONFIG_NAME: &str = "petridish.yaml";

pub trait Repository: Display {
    fn kind(&self) -> &'static str;
    fn determine_repo_dir(&self) -> PathBuf;
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

pub fn try_new_repository(repo: &str) -> Result<Box<dyn Repository>> {
    let ret: Box<dyn Repository>;

    if repo.ends_with(".git")
        && (repo.starts_with("https://") || repo.starts_with("http://") || repo.starts_with("git@"))
    {
        ret = Box::new(Git { uri: repo.into() });
    } else {
        ret = Box::new(Directory {
            path: PathBuf::from(repo.to_string()),
        });
    }

    ret.validate()?;
    Ok(ret)
}

struct Directory {
    path: PathBuf,
}

impl Repository for Directory {
    fn kind(&self) -> &'static str {
        "directory"
    }

    fn determine_repo_dir(&self) -> PathBuf {
        self.path.clone()
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
        assert_eq!(repo.determine_repo_dir(), PathBuf::from("/a/b"))
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

trait Cached {
    fn md5(&self) -> String;
    fn cached_repo(&self) -> PathBuf {
        let md5 = self.md5();
        let cache_dir = get_cache_dir();

        cache_dir.join(md5)
    }
}

/// Returns the path to the user's repository cache directory.
///
///
/// |Platform | Example                                                         |
/// | ------- | --------------------------------------------------------------- |
/// | Linux   | /home/alice/.config/petridish/repositories                      |
/// | macOS   | /Users/Alice/Library/Application Support/petridish/repositories |
/// | Windows | C:\Users\Alice\AppData\Roaming\petridish\repositories           |
fn get_cache_dir() -> PathBuf {
    cache_dir().unwrap().join("petridish/repositories")
}

struct Git {
    uri: String,
}

impl Cached for Git {
    fn md5(&self) -> String {
        format!("{:x}", md5::compute(&self.uri))
    }
}

impl Display for Git {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uri)
    }
}

impl Repository for Git {
    fn kind(&self) -> &'static str {
        "git"
    }

    fn determine_repo_dir(&self) -> PathBuf {
        self.cached_repo()
    }

    fn sync(&self) -> Result<()> {
        let cached_repo = self.cached_repo();
        if !cached_repo.exists() {
            fs::create_dir_all(&cached_repo).unwrap();

            // clone repo to cache dir
            Command::new("git")
                .args(["clone", "-q", &self.uri, cached_repo.to_str().unwrap()])
                .status()
                .map_err(|e| Error::Repo {
                    name: self.to_string(),
                    error: e.to_string(),
                })?;
        } else {
            // pull repo
            Command::new("git")
                .args(["pull", "-q"])
                .current_dir(&cached_repo)
                .status()
                .map_err(|e| Error::Repo {
                    name: self.to_string(),
                    error: e.to_string(),
                })?;
        }

        Ok(())
    }
}
