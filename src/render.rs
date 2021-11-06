use std::{
    fs::{self, create_dir_all},
    path::{Path, PathBuf},
};

use miette::Diagnostic;
use tempdir::TempDir;
use tera::Context;
use tera::Tera;
use thiserror::Error;
use walkdir::WalkDir;

pub struct Render {
    template_dir: PathBuf,
    entry_dir_name: String,
    context: Context,
    output_dir: PathBuf,
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
        entry_dir_name: &str,
        output_dir: &Path,
        context: Context,
    ) -> Result<Self, RenderError> {
        let entry_dir = &template_dir.join(&entry_dir_name);
        if !entry_dir.exists() {
            return Err(RenderError::EntryDirNotExists(entry_dir.into()));
        }

        Ok(Self {
            template_dir: template_dir.into(),
            entry_dir_name: entry_dir_name.into(),
            context,
            output_dir: output_dir.into(),
        })
    }
}

impl Render {
    pub fn render(&self) -> Result<(), RenderError> {
        let tmp_dir = TempDir::new("template").unwrap();
        let mut tera = Tera::default();
        for entry in WalkDir::new(&self.template_dir.join(&self.entry_dir_name))
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|p| p.file_type().is_file())
        {
            let render_entry_path = PathBuf::from(
                &tera
                    .render_str(&entry.path().display().to_string(), &self.context)
                    .map_err(|e| RenderError::RenderFailed(e.to_string()))?,
            );

            let tmp_entry_path = {
                &tmp_dir
                    .path()
                    .join(render_entry_path.strip_prefix(&self.template_dir).unwrap())
            };
            if let Some(parent_dir) = tmp_entry_path.parent() {
                if !parent_dir.exists() {
                    create_dir_all(parent_dir).unwrap();
                }
            }
            let template_content = fs::read_to_string(entry.path()).unwrap();
            fs::write(tmp_entry_path, {
                &tera
                    .render_str(&template_content, &self.context)
                    .map_err(|e| RenderError::RenderFailed(e.to_string()))?
            })
            .unwrap();
        }

        let rendered_entry_name = tera
            .render_str(&self.entry_dir_name, &self.context)
            .map_err(|e| RenderError::RenderFailed(e.to_string()))?;
        let rendered_entry_path = tmp_dir.path().join(&rendered_entry_name);
        if !self.output_dir.exists() {
            fs::create_dir_all(&self.output_dir).unwrap();
        }
        let output = self.output_dir.join(&rendered_entry_name);
        fs::rename(rendered_entry_path, output).unwrap();
        tmp_dir.close().unwrap();
        Ok(())
    }
}
