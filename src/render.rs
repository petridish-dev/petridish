use std::{collections::HashMap, fs, path::PathBuf};

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
    pub fn new(
        template_path: impl Into<PathBuf>,
        entry_dir_name: &str,
        output_path: impl Into<PathBuf>,
        context: Context,
    ) -> Self {
        Self {
            template_path: template_path.into(),
            entry_dir_name: entry_dir_name.into(),
            output_path: output_path.into(),
            context,
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
            file_contents.insert(relative_path, rendered_content);
        }

        // dump files
        for (relative_path, rendered_content) in file_contents {
            let output_path = self.output_path.join(relative_path);
            let parent = output_path.parent().unwrap();
            if !parent.exists() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(output_path, rendered_content).unwrap();
        }

        Ok(())
    }
}
