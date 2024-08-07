use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub struct Mqtt {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub keep_alive_intervall_seconds: u64,
    pub capacity: usize,
    pub user: Option<String>,
    pub password: Option<String>,
}

impl Default for Mqtt {
    fn default() -> Self {
        Self {
            id: String::from("mqtt-relay"),
            host: String::from("localhost"),
            port: 1883,
            keep_alive_intervall_seconds: 60,
            capacity: 10,
            user: None,
            password: None,
        }
    }
}