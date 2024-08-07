use std::{sync::{Arc, Mutex}, time::Duration};

use log::{debug, info, warn};
use notify::INotifyWatcher;
use notify_debouncer_mini::{new_debouncer_opt, Config, DebouncedEvent, DebouncedEventKind, Debouncer};
use rumqttc::{Client, QoS};

use crate::mappings::MappingConfig;

pub fn watch_for_fs_changes(mutex: Arc<Mutex<(Client, MappingConfig)>>) -> Result<Debouncer<INotifyWatcher>, String> {
    let clone = mutex.clone();
    let config = Config::default().with_timeout(Duration::from_secs(1)).with_batch_mode(false);
    let mut watcher = new_debouncer_opt(config, move |res: Result<Vec<DebouncedEvent>, notify::Error>| {
        match res {
            Ok(events) => {
                if events.iter().any(|e| e.kind == DebouncedEventKind::Any){
                    info!("Detected change in file system, reloading");
                    reload(mutex.clone());
                }
            },
            Err(e) => { warn!("Problem while watching for file system changes: {:?}", e);},
        };
    }).map_err(|err| format!("Failed to create file system watcher: {}", err))?;

    let path = &clone.lock().unwrap().1.path;
    watcher.watcher().watch(path, notify::RecursiveMode::Recursive)
        .map_err(|err| format!("Failed to start watching for file system changes: {}", err))?;

    info!("Watching for file system changes at '{}'", path.display());

    Ok(watcher)
}

pub fn reload(mutex: Arc<Mutex<(Client, MappingConfig)>>) {
    match mutex.lock() {
        Ok(mut guard) => {
            debug!("Got lock on client & mappings, reloading now.");
            guard.1.reload();
            
            // Start listening to messsaages
            let _ = guard.0.unsubscribe("#");
            for topic in guard.1.get_topics() {
                debug!("subscribing to {}", topic);
                guard.0.subscribe(topic, QoS::AtMostOnce).unwrap();
            }        
        },
        Err(err) => {
            warn!("Failed to reload: Cannot get lock: {}", err);
        },
    }    
}