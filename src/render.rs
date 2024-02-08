use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use tera::Context;
use tera::Tera;
use walkdir::WalkDir;

use crate::error::{Error, Result};

pub struct Render {
    template_path: PathBuf,
    entry_dir_name: String,
    output_path: PathBuf,
    context: Context,
    overwrite_if_exists: bool,
    skip_if_exists: bool,
    exclude_render_paths: Vec<String>,
}

impl Render {
    pub fn new(
        template_path: impl Into<PathBuf>,
        entry_dir_name: &str,
        output_path: impl Into<PathBuf>,
        context: Context,
        overwrite_if_exists: bool,
        skip_if_exists: bool,
        exclude_render_paths: Vec<String>,
    ) -> Self {
        let mut tera = Tera::default();
        let exclude_render_paths = exclude_render_paths
            .into_iter()
            .map(|p| {
                tera.render_str(&format!("{}/{}", entry_dir_name, p), &context)
                    .unwrap()
            })
            .collect();

        Self {
            template_path: template_path.into(),
            entry_dir_name: entry_dir_name.into(),
            output_path: output_path.into(),
            context,
            overwrite_if_exists,
            skip_if_exists,
            exclude_render_paths,
        }
    }
}

impl Render {
    pub fn render(&self) -> Result<()> {
        let mut tera = Tera::default();
        let mut file_contents = HashMap::new();

        // first render templates into file_contents
        let template_entry_path = self.template_path.join(&self.entry_dir_name);
        for entry in WalkDir::new(&template_entry_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|p| p.file_type().is_file() || p.file_type().is_symlink())
        {
            let relative_path = entry
                .path()
                .display()
                .to_string()
                .trim_start_matches(&self.template_path.display().to_string())
                .trim_start_matches('/') // for unix
                .trim_start_matches('\\') // for windows
                .to_string();

            let relative_path = tera.render_str(&relative_path, &self.context)?;
            let dest_path = self.output_path.join(&relative_path);
            if entry.path_is_symlink() {
                if !dest_path.parent().unwrap().exists() {
                    fs::create_dir_all(dest_path.parent().unwrap()).unwrap();
                }
                symlink(&fs::read_link(entry.path()).unwrap(), dest_path);
                continue;
            }

            let template_content = fs::read_to_string(entry.path()).unwrap();

            // check whether relative path is in exclude_render_paths
            if self
                .exclude_render_paths
                .iter()
                .any(|p| relative_path.eq(p))
            {
                file_contents.insert(dest_path, template_content);
            } else {
                let rendered_content = tera.render_str(&template_content, &self.context)?;
                file_contents.insert(dest_path, rendered_content);
            }
        }

        if !self.overwrite_if_exists && !self.skip_if_exists {
            // check whether dest path exists
            for dest_path in file_contents.keys() {
                if dest_path.exists() {
                    return Err(Error::CannotOverwriteContent(dest_path.clone()));
                }
            }
        }

        // dump files
        for (dest_path, rendered_content) in file_contents {
            let parent = dest_path.parent().unwrap();
            if !parent.exists() {
                fs::create_dir_all(parent).unwrap();
            }
            if !dest_path.exists() || self.overwrite_if_exists {
                fs::write(dest_path, rendered_content).unwrap();
            }
        }

        Ok(())
    }
}

#[cfg(windows)]
fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) {
    std::os::windows::fs::symlink_file(original, link).unwrap()
}

#[cfg(unix)]
fn symlink<P: AsRef<Path>, Q: AsRef<Path>>(original: P, link: Q) {
    std::os::unix::fs::symlink(original, link).unwrap()
}
