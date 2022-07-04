use std::{collections::HashMap, fs, path::PathBuf};

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
}

impl Render {
    pub fn new(
        template_path: impl Into<PathBuf>,
        entry_dir_name: &str,
        output_path: impl Into<PathBuf>,
        context: Context,
        overwrite_if_exists: bool,
        skip_if_exists: bool,
    ) -> Self {
        Self {
            template_path: template_path.into(),
            entry_dir_name: entry_dir_name.into(),
            output_path: output_path.into(),
            context,
            overwrite_if_exists,
            skip_if_exists,
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
            .filter(|p| p.file_type().is_file())
        {
            let template_content = fs::read_to_string(entry.path()).unwrap();
            let rendered_content = tera.render_str(&template_content, &self.context)?;
            let relative_path = entry
                .path()
                .display()
                .to_string()
                .trim_start_matches(&self.template_path.display().to_string())
                .trim_start_matches('/')
                .to_string();
            let relative_path = tera.render_str(&relative_path, &self.context)?;
            let dest_path = self.output_path.join(relative_path);
            file_contents.insert(dest_path, rendered_content);
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
