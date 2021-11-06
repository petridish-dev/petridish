use std::fs;
use std::path::PathBuf;
use std::process::Command;

use dirs::cache_dir;
use md5;
use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum SourceError {
    #[error("Template dir '{0}' does not exist")]
    DirNotExists(String),

    #[error("Config '{0}/petridish.yaml' does not exist")]
    ConfigNotExists(PathBuf),

    #[error("Update template '{0}' failed: {1}")]
    UpdateTemplateFailed(String, String),
}

fn get_cache_dir() -> PathBuf {
    cache_dir().unwrap().join("petridish/templates")
}

pub enum SourceStatus {
    NotExists,
    OutOfDate,
    Latest,
}

pub trait TemplateSource {
    fn get_status(&self) -> Result<SourceStatus, SourceError> {
        Ok(SourceStatus::Latest)
    }
    fn update(&self) -> Result<(), SourceError> {
        Ok(())
    }
    fn get_template(&self) -> Result<PathBuf, SourceError>;
    fn get_md5(&self) -> String {
        "".into()
    }
    fn get_config(&self) -> Result<PathBuf, SourceError> {
        let template = self.get_template()?;

        for config_name in ["petridish.yaml", "petridish.yml"] {
            let config = template.join(config_name);
            if config.exists() {
                return Ok(config);
            }
        }
        Err(SourceError::ConfigNotExists(template))
    }
    fn get_cached_source(&self) -> PathBuf {
        get_cache_dir().join(&self.get_md5())
    }
}

pub fn new_source(template: &str) -> Result<Box<dyn TemplateSource>, SourceError> {
    if template.ends_with(".git")
        && (template.starts_with("https://")
            || template.starts_with("http://")
            || template.starts_with("git@"))
    {
        return Ok(Box::new(GitSource::new(template)?));
    }

    Ok(Box::new(DirSource::new(template)?))
}

#[derive(PartialEq)]
struct DirSource {
    template_dir: PathBuf,
}

impl DirSource {
    fn new(template: &str) -> Result<Self, SourceError> {
        let template_dir = PathBuf::from(template);
        if !template_dir.exists() {
            return Err(SourceError::DirNotExists(template.to_string()));
        }

        Ok(DirSource { template_dir })
    }
}

impl TemplateSource for DirSource {
    fn get_template(&self) -> Result<PathBuf, SourceError> {
        Ok(self.template_dir.clone())
    }
}

#[cfg(test)]
mod dir_source_tests {
    use std::fs;

    use tempdir::TempDir;

    use super::*;

    #[test]
    fn it_new_source() {
        let template = TempDir::new("template").unwrap();
        let template = template.path().as_os_str().to_str().unwrap();
        let source = new_source(template).unwrap();

        assert_eq!(source.get_template().unwrap(), PathBuf::from(template));
    }

    #[test]
    fn it_new_source_failed() {
        let err = new_source("/a/b/c").err().unwrap();
        assert_eq!(format!("{}", err), "Template dir '/a/b/c' does not exist");
    }

    #[test]
    fn it_source_config() {
        let template = TempDir::new("template").unwrap();
        let template_path = &template.path().as_os_str().to_str().unwrap();
        let config = &template.path().join("petridish.yaml");
        fs::write(config.as_os_str().to_str().unwrap(), "").unwrap();

        let source = new_source(template_path).unwrap();
        assert_eq!(&source.get_config().unwrap(), config);
    }

    #[test]
    fn it_source_config_not_exists() {
        let template = TempDir::new("template").unwrap();
        let template_path = &template.path().as_os_str().to_str().unwrap();
        let source = new_source(template_path).unwrap();
        let err = source.get_config().err().unwrap();
        assert_eq!(
            format!("{}", err),
            format!("Config '{}/petridish.yaml' does not exist", template_path)
        );
    }
}

#[derive(PartialEq)]
struct GitSource {
    git_uri: String,
}

impl GitSource {
    fn new(git_uri: &str) -> Result<Self, SourceError> {
        Ok(Self {
            git_uri: git_uri.into(),
        })
    }
}

impl TemplateSource for GitSource {
    fn get_status(&self) -> Result<SourceStatus, SourceError> {
        let cache_source = self.get_cached_source();
        if !cache_source.exists() {
            return Ok(SourceStatus::NotExists);
        }

        Command::new("git")
            .args(["fetch", "-p", "-q"])
            .output()
            .map_err(|e| {
                SourceError::UpdateTemplateFailed(self.git_uri.to_string(), e.to_string())
            })?;

        let current_rev = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&cache_source)
            .output()
            .map_err(|e| {
                SourceError::UpdateTemplateFailed(self.git_uri.to_string(), e.to_string())
            })?;
        let current_rev = String::from_utf8(current_rev.stdout).unwrap();
        let remote_rev = Command::new("git")
            .args(["rev-parse", "origin/HEAD"])
            .current_dir(&cache_source)
            .output()
            .map_err(|e| {
                SourceError::UpdateTemplateFailed(self.git_uri.to_string(), e.to_string())
            })?;
        let remote_rev = String::from_utf8(remote_rev.stdout).unwrap();

        if current_rev != remote_rev {
            Ok(SourceStatus::OutOfDate)
        } else {
            Ok(SourceStatus::Latest)
        }
    }

    fn get_md5(&self) -> String {
        format!("{:x}", md5::compute(&self.git_uri))
    }

    fn get_template(&self) -> Result<PathBuf, SourceError> {
        let cached_source = self.get_cached_source();
        if !cached_source.exists() {
            let cached_dir = cached_source.parent().unwrap();
            if !cached_dir.exists() {
                fs::create_dir_all(cached_dir).unwrap();
            }

            Command::new("git")
                .args([
                    "clone",
                    "-q",
                    &self.git_uri,
                    cached_source.to_str().unwrap(),
                ])
                .output()
                .map_err(|e| {
                    SourceError::UpdateTemplateFailed(self.git_uri.to_string(), e.to_string())
                })?;
        }

        Ok(cached_source)
    }

    fn update(&self) -> Result<(), SourceError> {
        let cache_source = self.get_cached_source();
        Command::new("git")
            .args(["pull", "-q"])
            .current_dir(&cache_source)
            .output()
            .map_err(|e| {
                SourceError::UpdateTemplateFailed(self.git_uri.to_string(), e.to_string())
            })?;
        Ok(())
    }
}
