use std::{collections::HashMap, path::PathBuf};

use regex::Regex;

use crate::error::{Error, Result};

pub fn try_new_repo(uri: String, context: HashMap<String, String>) -> Result<Box<dyn Repository>> {
    if Git::check_match(&uri) {
        let repo = Git::try_new(uri, context)?;
        return Ok(Box::new(repo));
    }

    let local_repo = LocalPath::new(uri.into());
    Ok(Box::new(local_repo))
}

pub trait Repository {
    fn download(&self) -> Result<()>;
    fn repo_dir(&self) -> PathBuf;
}

#[derive(Debug, PartialEq)]
struct Git {
    uri: String,
    branch: Option<String>,
    auth: Option<Auth>,
}

impl Git {
    fn check_match(uri: &str) -> bool {
        uri.ends_with(".git") || Regex::new(r"^g(h|l).*:.*(\.git)?").unwrap().is_match(uri)
    }

    fn try_new(uri: String, context: HashMap<String, String>) -> Result<Self> {
        if Regex::new("^gh.*:.*").unwrap().is_match(&uri) {
            return Git::new_alias_git(uri, context, "gh", "github", "github.com");
        } else if Regex::new("^gl.*:.*").unwrap().is_match(&uri) {
            return Git::new_alias_git(uri, context, "gl", "gitlab", "gitlab.com");
        } else if uri.ends_with(".git") {
            return Git::new_git(uri, context);
        }

        Err(Error::InvalidRepo {
            kind: "git".into(),
            uri,
        })
    }

    fn new_git(uri: String, mut context: HashMap<String, String>) -> Result<Self> {
        let branch = context.remove("branch");
        let username = context.remove("username");
        let password = context.remove("password");

        if username.is_some() && password.is_none() {
            return Err(Error::AuthMissingPassword("git".into()));
        }
        if username.is_none() && password.is_some() {
            return Err(Error::AuthMissingUsername("git".into()));
        }
        let auth = if let (Some(username), Some(password)) = (username, password) {
            Some(Auth { username, password })
        } else {
            None
        };

        if uri.starts_with("https://") || uri.starts_with("http://") || uri.starts_with("git@") {
            Ok(Self { uri, branch, auth })
        } else {
            Err(Error::InvalidRepo {
                kind: "git".into(),
                uri,
            })
        }
    }

    #[allow(clippy::or_fun_call)]
    fn new_alias_git(
        alias_uri: String,
        mut context: HashMap<String, String>,
        alias: &str,
        provider: &str,
        provider_url: &str,
    ) -> Result<Self> {
        let head = alias_uri.split(':').collect::<Vec<&str>>()[0];
        let tail = alias_uri
            .trim_start_matches(&format!("{}:", head))
            .trim_end_matches(".git");

        let provider_url = context
            .remove(&format!("{}_provider", alias))
            .unwrap_or(provider_url.into());

        let url = if head == alias || head == format!("{}+https", alias) {
            format!("https://{}/{}.git", provider_url, tail)
        } else if head == format!("{}+http", alias) {
            format!("http://{}/{}.git", provider_url, tail)
        } else if head == format!("{}+ssh", alias) {
            format!("git@{}:{}.git", provider_url, tail)
        } else {
            return Err(Error::InvalidGitAliasRepo {
                alias: alias_uri,
                provider: provider.to_string(),
            });
        };

        Self::new_git(url, context)
    }
}

impl Repository for Git {
    fn download(&self) -> Result<()> {
        todo!()
    }

    fn repo_dir(&self) -> PathBuf {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
struct Auth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, PartialEq)]
struct LocalPath(PathBuf);

impl LocalPath {
    fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

impl Repository for LocalPath {
    fn download(&self) -> Result<()> {
        Ok(())
    }

    fn repo_dir(&self) -> PathBuf {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_git_repo() {
        let uri = "http://abc/hello.git";
        let repo = Git::try_new(uri.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "http://abc/hello.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_git_repo_with_specified_branch() {
        let uri = "http://abc/hello.git";
        let mut context = HashMap::new();
        context.insert("branch".to_string(), "dev".to_string());
        let repo = Git::try_new(uri.into(), context).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "http://abc/hello.git".into(),
                branch: Some("dev".into()),
                auth: None
            }
        );
    }

    #[test]
    fn test_git_repo_with_auth() {
        let uri = "http://abc/hello.git";
        let mut context = HashMap::new();
        context.insert("username".to_string(), "user1".to_string());
        context.insert("password".to_string(), "abc".to_string());
        let repo = Git::try_new(uri.into(), context).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "http://abc/hello.git".into(),
                branch: None,
                auth: Some(Auth {
                    username: "user1".into(),
                    password: "abc".into()
                })
            }
        );
    }

    #[test]
    fn test_invalid_git_repo() {
        let uri = "httpx://abc/hello.git";
        let err = Git::try_new(uri.into(), HashMap::new()).err().unwrap();
        assert_eq!(err.to_string(), "invalid git repo: httpx://abc/hello.git");
    }

    #[test]
    fn test_github() {
        let url = "gh:rust-lang/rust";
        let repo = Git::try_new(url.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "https://github.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_github_with_suffix() {
        let uri = "gh:rust-lang/rust";
        let repo = Git::try_new(uri.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "https://github.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_https_github() {
        let uri = "gh+https:rust-lang/rust";
        let repo = Git::try_new(uri.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "https://github.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_http_github() {
        let uri = "gh+http:rust-lang/rust";
        let repo = Git::try_new(uri.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "http://github.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_ssh_github() {
        let uri = "gh+ssh:rust-lang/rust";
        let repo = Git::try_new(uri.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "git@github.com:rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_gitlab() {
        let uri = "gl:rust-lang/rust";
        let repo = Git::try_new(uri.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "https://gitlab.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_https_gitlab() {
        let uri = "gl+https:rust-lang/rust";
        let repo = Git::try_new(uri.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "https://gitlab.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_http_gitlab() {
        let uri = "gl+http:rust-lang/rust";
        let repo = Git::try_new(uri.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "http://gitlab.com/rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_ssh_gitlab() {
        let uri = "gl+ssh:rust-lang/rust";
        let repo = Git::try_new(uri.into(), HashMap::new()).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "git@gitlab.com:rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }

    #[test]
    fn test_override_git_provider() {
        let uri = "gl+ssh:rust-lang/rust";
        let mut context = HashMap::new();
        context.insert("gl_provider".to_string(), "gitlab.cn.com".to_string());
        let repo = Git::try_new(uri.into(), context).unwrap();
        assert_eq!(
            repo,
            Git {
                uri: "git@gitlab.cn.com:rust-lang/rust.git".into(),
                branch: None,
                auth: None
            }
        );
    }
}
