pub use mqtt::Mqtt;
use serde::Deserialize;

mod mqtt;
mod load;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
    pub mappings_dir: String,
    pub mqtt: Mqtt,

}

impl Default for Config {
    fn default() -> Self {
        Self { 
            mappings_dir: String::from("./mappings"),
            mqtt: Default::default(),
        }
    }
}

pub fn auto_load() -> Config {
    Config::auto_load()
}