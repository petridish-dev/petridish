use std::path::{Path, PathBuf};

pub trait TemplateSource {
    fn check_update(&self, cache_dir: &Path) -> bool;
    fn get(&self) -> &Path;
}

#[derive(PartialEq)]
struct DirSource {
    template_dir: PathBuf,
}

impl TemplateSource for DirSource {
    fn check_update(&self, _: &Path) -> bool {
        false
    }

    fn get(&self) -> &Path {
        &self.template_dir
    }
}

pub fn new_source(template: &str) -> Option<Box<dyn TemplateSource>> {
    return Some(Box::new(DirSource {
        template_dir: PathBuf::from(template),
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_new_source() {
        let source = new_source("/a/b/c").unwrap();

        assert!(!source.check_update(&PathBuf::from("/cache")));
        assert_eq!(source.get(), &PathBuf::from("/a/b/c"));
    }
}
