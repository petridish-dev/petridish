use dirs::cache_dir;
use std::{
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

pub struct Cache;

impl Cache {
    /// Returns the path to the user's repository cache directory.
    ///
    ///
    /// |Platform | Example                                                         |
    /// | ------- | --------------------------------------------------------------- |
    /// | Linux   | /home/alice/.cache/petridish/repositories                      |
    /// | macOS   | /Users/Alice/Library/Application Support/petridish/repositories |
    /// | Windows | C:\Users\Alice\AppData\Roaming\petridish\repositories           |
    pub fn cache_dir() -> PathBuf {
        cache_dir().unwrap().join("petridish/repositories")
    }

    pub fn get(name: &str) -> Option<PathBuf> {
        let path = Cache::cache_dir().join(name);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    pub fn add(source: &Path) {
        let cache_dir = Cache::cache_dir();
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir).unwrap();
        }

        let dest = cache_dir.join(source.file_name().unwrap().to_str().unwrap());
        if dest.exists() {
            fs::remove_dir_all(&dest).unwrap();
        }
        fs::rename(source, dest).unwrap();
    }

    pub fn list() -> Vec<PathBuf> {
        WalkDir::new(Cache::cache_dir())
            .max_depth(1)
            .into_iter()
            .skip(1)
            .filter_map(|e| e.ok())
            .filter(|p| p.file_type().is_dir())
            .map(|p| p.path().to_owned())
            .collect::<Vec<PathBuf>>()
    }
}
