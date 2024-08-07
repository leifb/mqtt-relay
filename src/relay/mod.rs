use std::{path::PathBuf, sync::Arc, thread::{self}, time::Duration};

use fs_watch::  FileSystemWatcher;
use log::{debug, info, warn};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, Publish, QoS};
use tokio::sync::Mutex;

use crate::{config::{Config, Mqtt}, mappings::MappingConfig};

mod fs_watch;

pub struct Relay {
    client: AsyncClient,
    connection: EventLoop,
    mappings: MappingConfig,
}

impl Relay {
    pub async fn run(config: Config) {
        // Read mappings
        let mappings = MappingConfig::create(PathBuf::from(config.mappings_dir));

        // Create MQTT client
        let (client, connection)  = create_mqtt_client(config.mqtt);

        let relay = Relay {
            mappings,
            client,
            connection,
        };

        relay.start().await
    }

    async fn start(mut self) {
        let mutex = Arc::new(Mutex::new((self.client, self.mappings)));

        // Create the watcher what reloads mapping configs when they change
        let watcher = FileSystemWatcher::new(mutex.clone());

        // Initial "reload" to start up
        watcher.reload().await
            .expect("Failed loading mapping configs");

        tokio::spawn(async move {
            watcher.start().await;
        });          

        // Log that we are ready
        info!("Startup complete");

        // Main Loop
        loop {
            let notification = self.connection.poll().await;
        
            match notification {
                Ok(event) => {
                    let mutex = mutex.clone();
                    let mutex = mutex.lock().await;
                    on_event(event, &mutex.1, &mutex.0).await;
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


fn create_mqtt_client(config: Mqtt) -> (AsyncClient, EventLoop) {
    let mut mqttoptions = MqttOptions::new(config.id, config.host, config.port);
    mqttoptions.set_keep_alive(Duration::from_secs(config.keep_alive_intervall_seconds));

    if config.user.is_some() && config.password.is_some() {
        mqttoptions.set_credentials(config.user.unwrap(), config.password.unwrap());
    }

    AsyncClient::new(mqttoptions, config.capacity)
}


async fn on_event(event: Event, mappings: &MappingConfig, client: &AsyncClient) {
    match event {
        Event::Incoming(incoming) => {
            match incoming {
                Packet::Publish(message) => {
                    let _ = on_message(message, mappings, client)
                        .await
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

async fn on_message(message: Publish, mappings: &MappingConfig, client: &AsyncClient) -> Result<(), String> {
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
        ).await.inspect_err(|error| {
            warn!("Failed to publish to {}: {}", message.topic, error);
        });
    }

    Ok(())
}
