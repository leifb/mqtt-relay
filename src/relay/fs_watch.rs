use std::{sync::Arc, time::Duration};

use log::{debug, info, trace, warn};
use notify::{INotifyWatcher, Watcher};
use rumqttc::{AsyncClient, QoS};
use tokio::{sync::{mpsc::{channel, Receiver}, Mutex}, time::{sleep, Instant}};


use crate::mappings::MappingConfig;


pub struct FileSystemWatcher {
    receiver: Receiver<Instant>,
    client_and_mappings: Arc<Mutex<(AsyncClient, MappingConfig)>>,
    watcher: INotifyWatcher,
}

impl FileSystemWatcher {
    pub fn new(client_and_mappings: Arc<Mutex<(AsyncClient, MappingConfig)>>) -> FileSystemWatcher {
        let (sender, receiver) = channel::<Instant>(10);

        FileSystemWatcher {
            receiver,
            client_and_mappings,
            watcher: notify::recommended_watcher(move |res: Result<_,_>| {
                match res {
                    Ok(event) => {
                        trace!("File system event: {:?}", event);
                        sender.blocking_send(Instant::now()).unwrap();
                    },
                    Err(e) => { warn!("Problem while watching for file system changes: {:?}", e);},
                };
            }).expect("Failed to create file system watcher")
        }
    }

    /// Starts watching for changes in the file system & reloading when these happens.
    /// Note that this method does never return.
    pub async fn start(mut self) {
        let path = self.client_and_mappings.lock().await.1.path.clone();
        info!("Watching for file system changes at '{}'", path.display());

        // Start the watcher     
        {
            self.watcher.watch(&path, notify::RecursiveMode::Recursive)
                .expect("Failed to start watching for file system changes");
        }  

        // Start receiving events     
        let mut time_last_reload = Instant::now();   
        while let Some(time_of_change) = self.receiver.recv().await {
            // Don't do anything if file system event is older than last reload
            if time_of_change < time_last_reload {
                continue;
            }
            time_last_reload = self.queue_reload().await;
        }
    }

    /// Waits a little while and then reloads the mapping configs.
    /// If the loading fails (usually because the mappings / client mutex is locked),
    /// tries again after another delay.
    async fn queue_reload(&self) -> Instant {
        debug!("Detected change in file system, reloading");
        let mut retries = 10;

        loop {
            sleep(Duration::from_millis(200)).await;
            let time_reload = Instant::now();
            let result = self.reload().await;

            // Return time just before reloading when all ok
            if result.is_ok() {
                info!("Reloaded mappings");
                break time_reload;
            }

            // Log if reloading did not work
            // TODO change to debug! if it happens too often
            warn!("{}", result.unwrap_err());

            // Cancel if no more retries left
            retries -= 1;
            if retries <= 0 {
                warn!("Failed to reload mappings too often, canceling reloading");
                break time_reload;
            }
        }
    }


    ///
    pub async fn reload(&self) -> Result<(), String> {
        let mut guard = self.client_and_mappings.try_lock()
            .map_err(|err| format!("Failed to reload: Cannot get lock: {}", err))?;
        
        debug!("Got lock on client & mappings, reloading now.");
        guard.1.reload();
        
        // Start listening to messsaages
        guard.0.unsubscribe("#").await.unwrap();
        for topic in guard.1.get_topics() {
            debug!("subscribing to {}", topic);
            guard.0.subscribe(topic, QoS::AtMostOnce).await.unwrap();
        }
            
        Ok(())
    }
}
