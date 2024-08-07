use std::{collections::HashMap, path::PathBuf};

use mapping::Mapping;

mod load;
mod mapping;

pub struct MappingConfig {
    pub path: PathBuf,
    mappings: HashMap<String, Mapping>,
}

impl MappingConfig {
    pub fn get_mapping(&self, key: &String) -> Option<&Mapping> {
        self.mappings.get(key)
    }

    pub fn get_topics(&self) -> Vec<&String> {
        self.mappings.iter()
            .map(|mapping| mapping.0)
            .collect()
    }
}