use dirs::cache_dir;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Cache;

/// Returns the path to the user's repository cache directory.
///
///
/// |Platform | Example                                                         |
/// | ------- | --------------------------------------------------------------- |
/// | Linux   | /home/alice/.config/petridish/repositories                      |
/// | macOS   | /Users/Alice/Library/Application Support/petridish/repositories |
/// | Windows | C:\Users\Alice\AppData\Roaming\petridish\repositories           |
fn get_cache_dir() -> PathBuf {
    cache_dir().unwrap().join("petridish/repositories")
}

impl Cache {
    pub fn get(name: &str) -> Option<PathBuf> {
        let path = get_cache_dir().join(name);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    pub fn add(source: &Path) {
        let cache_dir = get_cache_dir();
        if !cache_dir.exists() {
            fs::create_dir_all(cache_dir).unwrap();
        }

        let dest = get_cache_dir().join(source.file_name().unwrap().to_str().unwrap());
        if dest.exists() {
            fs::remove_dir_all(&dest).unwrap();
        }
        fs::rename(source, dest).unwrap();
    }
}
