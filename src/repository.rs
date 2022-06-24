use std::{collections::HashMap, path::PathBuf};

use regex::Regex;

use crate::error::{Error, Result};

#[derive(Debug, PartialEq)]
pub enum Repository {
    LocalDir(PathBuf),
    Git {
        url: String,
        branch: Option<String>,
        auth: Option<GitAuth>,
    },
}

#[derive(Debug, PartialEq)]
pub struct GitAuth {
    pub username: String,
    pub password: String,
}

impl Repository {
    pub fn try_new(repo: String, context: HashMap<String, String>) -> Result<Self> {
        if repo.ends_with(".git") {
            return Repository::new_git(repo, context);
        } else if Regex::new("^gh.*:.*").unwrap().is_match(&repo) {
            return Repository::new_alias_git(repo, context, "gh", "github", "github.com");
        } else if Regex::new("^gl.*:.*").unwrap().is_match(&repo) {
            return Repository::new_alias_git(repo, context, "gl", "gitlab", "gitlab.com");
        }

        // local path
        Ok(Self::LocalDir(repo.into()))
    }

    fn new_git(repo: String, mut context: HashMap<String, String>) -> Result<Self> {
        let branch = context.remove("branch");
        let username = context.remove("username");
        let password = context.remove("password");
        if username.is_some() && password.is_none() {
            return Err(Error::InvalidRepo("git `password` is not provided".into()));
        }
        if username.is_none() && password.is_some() {
            return Err(Error::InvalidRepo("git `username` is not provided".into()));
        }
        let auth = if let (Some(username), Some(password)) = (username, password) {
            Some(GitAuth { username, password })
        } else {
            None
        };

        if repo.starts_with("https://") || repo.starts_with("http://") || repo.starts_with("git@") {
            Ok(Repository::Git {
                url: repo,
                branch,
                auth,
            })
        } else {
            Err(Error::InvalidRepo(format!(
                "'{}' is invalid git repo",
                repo
            )))
        }
    }

    #[allow(clippy::or_fun_call)]
    fn new_alias_git(
        alias_repo: String,
        mut context: HashMap<String, String>,
        alias: &str,
        provider: &str,
        provider_url: &str,
    ) -> Result<Self> {
        let head = alias_repo.split(':').collect::<Vec<&str>>()[0];
        let tail = alias_repo.strip_prefix(&format!("{}:", head)).unwrap();

        let provider_url = context
            .remove(&format!("{}_alias", alias))
            .unwrap_or(provider_url.into());

        let url = if head == alias || head == format!("{}+https", alias) {
            format!("https://{}/{}.git", provider_url, tail)
        } else if head == format!("{}+http", alias) {
            format!("http://{}/{}.git", provider_url, tail)
        } else if head == format!("{}+ssh", alias) {
            format!("git@{}:{}.git", provider_url, tail)
        } else {
            return Err(Error::InvalidRepo(format!(
                "{} is invalid `{}` repo",
                alias_repo, provider
            )));
        };

        Self::new_git(url, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_git_repo() {
        let url = "http://abc/hello.git";
        let repo = Repository::try_new(url.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Repository::Git {
                url: "http://abc/hello.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_git_repo_with_specified_branch() {
        let url = "http://abc/hello.git";
        let mut context = HashMap::new();
        context.insert("branch".to_string(), "dev".to_string());
        let repo = Repository::try_new(url.into(), context).unwrap();
        assert_eq!(
            repo,
            Repository::Git {
                url: "http://abc/hello.git".into(),
                branch: Some("dev".into()),
                auth: None
            }
        );
    }

    #[test]
    fn test_git_repo_with_auth() {
        let url = "http://abc/hello.git";
        let mut context = HashMap::new();
        context.insert("username".to_string(), "user1".to_string());
        context.insert("password".to_string(), "abc".to_string());
        let repo = Repository::try_new(url.into(), context).unwrap();
        assert_eq!(
            repo,
            Repository::Git {
                url: "http://abc/hello.git".into(),
                branch: None,
                auth: Some(GitAuth {
                    username: "user1".into(),
                    password: "abc".into()
                })
            }
        );
    }

    #[test]
    fn test_invalid_git_repo() {
        let url = "httpx://abc/hello.git";
        let err = Repository::try_new(url.into(), HashMap::new())
            .err()
            .unwrap();
        assert_eq!(
            err,
            Error::InvalidRepo("'httpx://abc/hello.git' is invalid git repo".into())
        );
    }

    #[test]
    fn test_github() {
        let url = "gh:rust-lang/rust";
        let repo = Repository::try_new(url.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Repository::Git {
                url: "https://github.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_https_github() {
        let url = "gh+https:rust-lang/rust";
        let repo = Repository::try_new(url.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Repository::Git {
                url: "https://github.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_http_github() {
        let url = "gh+http:rust-lang/rust";
        let repo = Repository::try_new(url.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Repository::Git {
                url: "http://github.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_ssh_github() {
        let url = "gh+ssh:rust-lang/rust";
        let repo = Repository::try_new(url.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Repository::Git {
                url: "git@github.com:rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_gitlab() {
        let url = "gl:rust-lang/rust";
        let repo = Repository::try_new(url.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Repository::Git {
                url: "https://gitlab.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_https_gitlab() {
        let url = "gl+https:rust-lang/rust";
        let repo = Repository::try_new(url.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Repository::Git {
                url: "https://gitlab.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_http_gitlab() {
        let url = "gl+http:rust-lang/rust";
        let repo = Repository::try_new(url.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Repository::Git {
                url: "http://gitlab.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_ssh_gitlab() {
        let url = "gl+ssh:rust-lang/rust";
        let repo = Repository::try_new(url.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Repository::Git {
                url: "git@gitlab.com:rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }
}
