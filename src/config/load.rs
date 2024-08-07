use std::{fs::read_to_string, path::PathBuf};

use super::Config;

impl Config {
    pub fn auto_load() -> Self {
        // For now, location is hardcoded
        let path = PathBuf::from("./config.yaml");

        // If config file exists, load it
        if path.is_file() {
            return Config::load_from_config_file(path);
        }

        // Use default otherwise
        Default::default()
    }

    fn load_from_config_file(path: PathBuf) -> Self {
        let file_conntent = read_to_string(path.clone())
            .expect("Failed to read config file");

        serde_yml::from_str(&file_conntent)
            .expect("Failed to parse config file")
    }
}