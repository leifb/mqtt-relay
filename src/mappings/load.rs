use std::{collections::HashMap, fs::{read_dir, read_to_string}, path::PathBuf};

use log::{debug, warn};
use yaml_rust2::YamlLoader;

use super::{mapping::Mapping, MappingConfig};

impl MappingConfig {
    pub fn create(path: PathBuf) -> Self {
        MappingConfig {
            path: path.clone(),
            mappings: HashMap::new(),
        }
    }

    pub fn reload(&mut self) {
        if !self.path.is_dir() {
            warn!("Failed to load mapping config: Missing directory {}", self.path.display());
            return;
        }

        let len = self.path.as_os_str().len();

        self.mappings = load_recursive(&self.path)
            .inspect_err(|err| warn!("Failed to load mappings: {}", err))
            .unwrap_or_default()
            .into_iter()
            .map(|(p, v)| (String::from(&p[len..]), v))
            .collect();
    }   
}



fn load_recursive(parent: &PathBuf) -> Result<Vec<(String, Mapping)>, String> {
    let directory_contents = read_dir(parent)
        .map_err(|err| format!("Problem while loading mappings, skipping: {}", err))?;

    let mappings = directory_contents
        .filter_map(|entry| entry
            .inspect_err(|err| warn!("Problem while loading mappings, skipping: {}", err))
            .ok()
        )
        .flat_map(|entry| {              
                let t = entry.file_type().unwrap();

                // Recursive call if entry is a directory
                if t.is_dir() {
                    return load_recursive(&entry.path())
                        .inspect_err(|err| warn!("{}", err))
                        .unwrap_or(vec![]);
                }

                // Ignore anything that is not a directory nor a file
                if !t.is_file() {
                    return vec![];
                }

                // Ignore anything that is not a yaml
                if entry.path().extension().unwrap_or_default() != "yaml" {
                    return vec![];
                }
                
                // Get the key for this config file
                let key = entry.path().with_extension("");
                let key = key.display().to_string();
                let key = String::from(&key[1..]);

                // Read the actual file
                return read_config(entry.path())
                    .inspect_err(|err| warn!("Failed to read mapping file: {}", err))
                    .map(|mapping| vec![(key, mapping)])
                    .unwrap_or_default();
        }).collect();

    Ok(mappings)
}

fn read_config(path: PathBuf) -> Result<Mapping, String> {
    debug!("Reading mapping file {}", path.display());

    // read data
    let file_conntent = read_to_string(path.clone())
        .map_err(|err| format!("Failed to read file: {}", err))?;

    // parse yaml
    let doc = YamlLoader::load_from_str(&file_conntent)
        .map_err(|err| format!("Failed to parse yaml: {}", err))?
        .pop()
        .ok_or(format!("Config did not contain a document: {}", path.display()))?;
        
    Mapping::from_yaml(doc)
}