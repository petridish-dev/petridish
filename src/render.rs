use std::{
    fs::{create_dir_all, DirEntry},
    path::{Path, PathBuf},
};

use miette::Diagnostic;
use tempdir::TempDir;
use tera::Context;
use tera::Tera;
use thiserror::Error;
use walkdir::WalkDir;

pub struct Render {
    entry_dir: PathBuf,
    context: Context,
    output: PathBuf,
}

#[derive(Error, Debug, Diagnostic)]
pub enum RenderError {
    #[error("Entry dir '{0}' does not exist")]
    EntryDirNotExists(PathBuf),

    #[error("{0}")]
    RenderFailed(String),
}

impl Render {
    pub fn try_new(
        template_dir: &Path,
        entry_dir: String,
        output: &Path,
        context: Context,
    ) -> Result<Self, RenderError> {
        let entry_dir = template_dir.join(entry_dir);
        if !entry_dir.exists() {
            return Err(RenderError::EntryDirNotExists(entry_dir));
        }

        Ok(Self {
            entry_dir,
            context,
            output: output.into(),
        })
    }
}

impl Render {
    pub fn render(&self) -> Result<(), RenderError> {
        let tmp_dir = TempDir::new("template").unwrap();
        let mut tera = Tera::default();
        for entry in WalkDir::new(&self.entry_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|p| p.file_type().is_file())
        {
            let render_entry_path = &tera
                .render_str(&entry.path().display().to_string(), &self.context)
                .map_err(|e| RenderError::RenderFailed(e.to_string()))?;
            let tmp_render_dir = &tmp_dir.path().join(render_entry_path).parent().unwrap();
            if !tmp_render_dir.exists() {
                // create_dir_all(tmp_render_dir);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use walkdir::WalkDir;

    #[test]
    fn it() {
        for entry in WalkDir::new("src")
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|p| p.file_type().is_file())
        {
            println!("{}", entry.path().display());
        }
    }
}
