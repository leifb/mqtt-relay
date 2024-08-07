use std::{path::PathBuf, sync::{Arc, Mutex}, thread, time::Duration};

use fs_watch::{reload, watch_for_fs_changes};
use log::{debug, info, warn};
use rumqttc::{Client, Connection, Event, MqttOptions, Packet, Publish, QoS};

use crate::{config::{Config, Mqtt}, mappings::MappingConfig};

mod fs_watch;

pub struct Relay {
    client: Client,
    connection: Connection,
    mappings: MappingConfig,
}

impl Relay {
    pub fn run(config: Config) {
        // Read mappings
        let mappings = MappingConfig::create(PathBuf::from(config.mappings_dir));

        // Create MQTT client
        let (client, connection)  = create_mqtt_client(config.mqtt);

        let relay = Relay {
            mappings,
            client,
            connection,
        };

        relay.start()
    }

    fn start(mut self) {
        let mutex = Arc::new(Mutex::new((self.client, self.mappings)));

        // Start listening for fs changes
        let _watcher = watch_for_fs_changes(mutex.clone())
            .expect("Failed to start watching for fs changes");

        // Initial "reload" to start up
        reload(mutex.clone());        

        // Log that we are ready
        info!("Startup complete");

        // Main Loop
        for notification in self.connection.iter() {
            match notification {
                Ok(event) => {
                    match mutex.clone().lock() {
                        Ok(mutex) => {
                            on_event(event, &mutex.1, &mutex.0);
                        },
                        Err(_) => todo!(),
                    }
                    
                },
                Err(error) => {
                    warn!("MQTT connection error: {:?}", error);
                    // Not a great solution, but this prevents spamming the logs
                    thread::sleep(Duration::from_secs(5));
                },
            }
        }
    }    
}


fn create_mqtt_client(config: Mqtt) -> (Client, Connection) {
    let mut mqttoptions = MqttOptions::new(config.id, config.host, config.port);
    mqttoptions.set_keep_alive(Duration::from_secs(config.keep_alive_intervall_seconds));

    if config.user.is_some() && config.password.is_some() {
        mqttoptions.set_credentials(config.user.unwrap(), config.password.unwrap());
    }

    Client::new(mqttoptions, config.capacity)
}


fn on_event(event: Event, mappings: &MappingConfig, client: &Client) {
    match event {
        Event::Incoming(incoming) => {
            match incoming {
                Packet::Publish(message) => {
                    let _ = on_message(message, mappings, client)
                        .inspect_err(|err| warn!("Failed to handle message: {}", err));
                },
                _ => {
                    debug!("Misc MQTT event: {:?}", incoming);
                }
            }
            
        },
        // Ignore outgoing events
        Event::Outgoing(_) => {},
    }
}

fn on_message(message: Publish, mappings: &MappingConfig, client: &Client) -> Result<(), String> {
    debug!("Received message on topic {}", message.topic);
    let mapping = mappings.get_mapping(&message.topic)
        .ok_or(format!("Could not find mapping for topic '{}'", message.topic))?;
    
    
    for message in mapping.matching_messages(&message) {
        debug!("[Relay] publishing to {}", message.topic);
        let _ = client.publish(
            message.topic.clone(),
            QoS::AtMostOnce,
            false,
            message.message.clone()
        ).inspect_err(|error| {
            warn!("Failed to publish to {}: {}", message.topic, error);
        });
    }

    Ok(())
}
