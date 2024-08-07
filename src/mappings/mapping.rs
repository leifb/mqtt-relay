use core::str;
use std::collections::HashMap;

use bytes::Bytes;
use rumqttc::Publish;
use yaml_rust2::Yaml;


#[derive(Debug)]
pub struct Mapping {
    pub conditions: HashMap<String, String>,
    pub messages: Vec<MappingMessage>,
}

impl Mapping {
    pub fn from_yaml(yaml: Yaml) -> Result<Mapping, String> {
        Ok(Mapping {
            conditions: parse_conditions(&yaml).unwrap_or_default(),
            messages: parse_messages(&yaml).unwrap_or_default(),
        })
    }

    pub fn matching_messages(&self, message: &Publish) -> Vec<&MappingMessage> {
        self.messages.iter()
            .filter(|mapping| self.is_mapping_matching(message, mapping))
            .collect()
    }

    fn is_mapping_matching(&self, message: &Publish, mapping: &MappingMessage) -> bool {
        match &mapping.condition {
            None => true,
            Some(condition) => match self.conditions.get(condition) {
                None => false,
                Some(condition) => {
                    is_condition_matching_payload(condition, &message.payload)
                        .unwrap_or(false)
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct MappingMessage {
    pub condition: Option<String>,
    pub topic: String,
    pub message: String,
}

impl MappingMessage {
    pub fn from_yaml(yaml: &Yaml) -> Option<MappingMessage> {
        Some(MappingMessage {
            condition: yaml["condition"].as_str().map(|c| c.to_string()),
            topic: yaml["topic"].as_str()?.to_string(),
            message: yaml["message"].as_str()?.to_string(),
        })
    }

    
}

fn parse_conditions(yaml: &Yaml) -> Option<HashMap<String, String>> {
    Some(yaml["conditions"]
        // Assume "conditions" is a map
        .as_hash()?
        // Iterate over key, value pairs
        .iter()
        // Take only the items where key and value are strings
        .filter_map(|(key, value)| match (key.as_str(), value.as_str()) {
            (Some(k), Some(v)) => Some((k, v)),
            _ => None
        })
        // Map &str to String
        .map(|(k, v)| (String::from(k), String::from(v)))
        // Build HashMap
        .collect()
    )
}

fn parse_messages(yaml: &Yaml) -> Option<Vec<MappingMessage>> {
    Some(yaml["messages"].as_vec()?
        .into_iter()
        .filter_map(MappingMessage::from_yaml)
        .collect()
    )
}

fn is_condition_matching_payload(condition: &String, payload: &Bytes) -> Option<bool> {
    let payload_as_str = str::from_utf8(payload.as_ref()).ok()?;
    let result = jq_rs::run(&condition, payload_as_str).ok()?;
    Some(result == "true\n")
}