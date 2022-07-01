use std::{
    fs::{self, create_dir_all},
    path::PathBuf,
};

use tempdir::TempDir;
use tera::Context;
use tera::Tera;
use walkdir::WalkDir;

use crate::error::Result;

pub struct Render {
    template_path: PathBuf,
    entry_dir_name: String,
    output_path: PathBuf,
    context: Context,
}

impl Render {
    pub fn try_new(
        template_path: impl Into<PathBuf>,
        entry_dir_name: &str,
        output_path: impl Into<PathBuf>,
        context: Context,
    ) -> Result<Self> {
        let template_dir: PathBuf = template_path.into();

        Ok(Self {
            template_path: template_dir,
            entry_dir_name: entry_dir_name.into(),
            output_path: output_path.into(),
            context,
        })
    }
}

impl Render {
    pub fn render(&self) -> Result<()> {
        let tmp_dir = TempDir::new("template").unwrap();
        let mut tera = Tera::default();

        // first render files
        let template_entry_path = self.template_path.join(&self.entry_dir_name);
        for entry in WalkDir::new(&template_entry_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|p| p.file_type().is_file())
        {
            let render_path = {
                &tmp_dir
                    .path()
                    .join(entry.path().strip_prefix(&self.template_path).unwrap())
            };
            if let Some(parent_dir) = render_path.parent() {
                if !parent_dir.exists() {
                    create_dir_all(parent_dir).unwrap();
                }
            }
            let template_content = fs::read_to_string(entry.path()).unwrap();
            fs::write(
                render_path,
                &tera.render_str(&template_content, &self.context)?,
            )
            .unwrap();
        }

        // second render directories
        let tmp_dir2 = TempDir::new("template2").unwrap();
        let rendered_file_paths = WalkDir::new(tmp_dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|p| p.file_type().is_file())
            .map(|v| v.path().display().to_string())
            .collect::<Vec<String>>();
        for rendered_file_path in rendered_file_paths {
            let rendered_path: PathBuf = tera
                .render_str(&rendered_file_path, &self.context)?
                .replace(
                    tmp_dir.path().to_str().unwrap(),
                    tmp_dir2.path().to_str().unwrap(),
                )
                .into();

            if let Some(parent) = rendered_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).unwrap();
                }
            }

            fs::rename(rendered_file_path, rendered_path).unwrap();
        }

        // move rendered project to output
        let rendered_entry_dir_name = tera.render_str(&self.entry_dir_name, &self.context)?;
        let tmp_entry_path = tmp_dir2.path().join(&rendered_entry_dir_name);
        if !self.output_path.exists() {
            fs::create_dir_all(&self.output_path).unwrap();
        }

        let output = self.output_path.join(&rendered_entry_dir_name);
        fs::rename(tmp_entry_path, output).unwrap();
        tmp_dir.close().unwrap();
        tmp_dir2.close().unwrap();
        Ok(())
    }
}
