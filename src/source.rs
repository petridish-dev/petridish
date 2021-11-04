use miette::Diagnostic;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum SourceError {
    #[error("Template dir '{0}' does not exist")]
    DirNotExists(String),

    #[error("Config '{0}/petridish.yaml' does not exist")]
    ConfigNotExists(PathBuf),
}

pub trait TemplateSource {
    fn check_update(&self, cache_dir: &Path) -> bool;
    fn get_template(&self) -> &Path;
    fn get_config(&self) -> Result<PathBuf, SourceError> {
        let template = self.get_template();

        for config_name in ["petridish.yaml", "petridish.yml"] {
            let config = template.join(config_name);
            if config.exists() {
                return Ok(config);
            }
        }
        Err(SourceError::ConfigNotExists(template.to_path_buf()))
    }
}

#[derive(PartialEq)]
struct DirSource {
    template_dir: PathBuf,
}

impl DirSource {
    fn new(template: &str) -> Result<Self, SourceError> {
        let template_dir = PathBuf::from(template);
        if !template_dir.exists() {
            Err(SourceError::DirNotExists(template.to_string()))?
        }

        Ok(DirSource { template_dir })
    }
}

impl TemplateSource for DirSource {
    fn check_update(&self, _: &Path) -> bool {
        false
    }

    fn get_template(&self) -> &Path {
        &self.template_dir
    }
}

pub fn new_source(template: &str) -> Result<Box<dyn TemplateSource>, SourceError> {
    return Ok(Box::new(DirSource::new(template)?));
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempdir::TempDir;

    use super::*;

    #[test]
    fn it_new_source() {
        let template = TempDir::new("template").unwrap();
        let template = template.path().as_os_str().to_str().unwrap();
        let source = new_source(template).unwrap();

        assert!(!source.check_update(&PathBuf::from("/cache")));
        assert_eq!(source.get_template(), &PathBuf::from(template));
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
