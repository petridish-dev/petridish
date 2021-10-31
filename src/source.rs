use miette::Diagnostic;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum SourceError {
    #[error("Template dir '{0}' does not exists")]
    #[diagnostic(code(source::not_exists))]
    DirNotExists(String),
}

pub trait TemplateSource {
    fn check_update(&self, cache_dir: &Path) -> bool;
    fn get(&self) -> &Path;
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

    fn get(&self) -> &Path {
        &self.template_dir
    }
}

pub fn new_source(template: &str) -> Result<Box<dyn TemplateSource>, SourceError> {
    return Ok(Box::new(DirSource::new(template)?));
}

#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn it_new_source() {
        let template = TempDir::new("template").unwrap();
        let template = template.path().as_os_str().to_str().unwrap();
        let source = new_source(template).unwrap();

        assert!(!source.check_update(&PathBuf::from("/cache")));
        assert_eq!(source.get(), &PathBuf::from(template));
    }

    #[test]
    fn it_new_source_failed() {
        let err = new_source("/a/b/c").err().unwrap();
        assert_eq!(format!("{}", err), "Template dir '/a/b/c' does not exists");
    }
}
