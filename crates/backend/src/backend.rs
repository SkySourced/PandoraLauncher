use std::{collections::HashMap, path::{Path, PathBuf}, sync::Arc, time::Duration};

use bridge::{handle::FrontendHandle, instance::InstanceID, message::{MessageToBackend, MessageToFrontend}};
use slab::Slab;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{directories::LauncherDirectories, instance::Instance, launch::Launcher, metadata::manager::MetadataManager};

pub fn start(send: FrontendHandle, recv: Receiver<MessageToBackend>) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("Failed to initialize Tokio runtime");

    let http_client = reqwest::ClientBuilder::new()
        // .connect_timeout(Duration::from_secs(5))
        .use_rustls_tls()
        .user_agent("PandoraLauncher/0.1.0 (https://github.com/Moulberry/PandoraLauncher)")
        .build().unwrap();

    let base_dirs = directories::BaseDirs::new().unwrap();
    let data_dir = base_dirs.data_dir();
    let launcher_dir = data_dir.join("PandoraLauncher");
    let directories = Arc::new(LauncherDirectories::new(launcher_dir));

    let meta = Arc::new(MetadataManager::new(http_client.clone(), runtime.handle().clone(), directories.metadata_dir.clone(), send.clone()));
    
    let (watcher_tx, watcher_rx) = tokio::sync::mpsc::channel::<notify_debouncer_full::DebounceEventResult>(64);
    let watcher = notify_debouncer_full::new_debouncer(Duration::from_millis(100), None, move |event| {
        let _ = watcher_tx.blocking_send(event);
    }).unwrap();
    
    let state = BackendState {
        recv,
        send: send.clone(),
        watcher,
        watcher_rx,
        http_client,
        meta: Arc::clone(&meta),
        watching: HashMap::new(),
        instances: Slab::new(),
        instance_by_path: HashMap::new(),
        instances_generation: 0,
        directories: Arc::clone(&directories),
        launcher: Launcher::new(meta, directories, send),
    };

    runtime.spawn(state.start());

    std::mem::forget(runtime);
}

pub enum WatchTarget {
    InstancesDir,
    InstanceDir {
        id: InstanceID,
    },
    InvalidInstanceDir,
    InstanceLevelDir {
        id: InstanceID,
    },
    InstanceSavesDir {
        id: InstanceID
    },
    ServersDat {
        id: InstanceID
    },
}

pub struct BackendState {
    pub recv: Receiver<MessageToBackend>,
    pub send: FrontendHandle,
    pub watcher: notify_debouncer_full::Debouncer<notify::RecommendedWatcher, notify_debouncer_full::RecommendedCache>,
    pub watcher_rx: Receiver<notify_debouncer_full::DebounceEventResult>,
    pub http_client: reqwest::Client,
    pub meta: Arc<MetadataManager>,
    pub watching: HashMap<Arc<Path>, WatchTarget>,
    pub instances: Slab<Instance>,
    pub instance_by_path: HashMap<PathBuf, InstanceID>,
    pub instances_generation: usize,
    pub directories: Arc<LauncherDirectories>,
    pub launcher: Launcher
}

impl BackendState {
    async fn start(mut self) {
        todo!();
        
        let _ = std::fs::create_dir_all(&self.directories.instances_dir);
        
        self.watch_filesystem(&self.directories.instances_dir.clone(), WatchTarget::InstancesDir).await;
        
        for entry in std::fs::read_dir(&self.directories.instances_dir).unwrap() {
            let Ok(entry) = entry else {
                eprintln!("Error reading directory in instances folder: {:?}", entry.unwrap_err());
                continue;
            };
            
            let success = self.load_instance_from_path(&entry.path(), true, false).await;
            if !success {
                self.watch_filesystem(&entry.path(), WatchTarget::InvalidInstanceDir).await;
            }
        }
        
        self.handle().await;
    }
    
    pub async fn watch_filesystem(&mut self, path: &Path, target: WatchTarget) {
        if self.watcher.watch(path, notify::RecursiveMode::NonRecursive).is_err() {
            self.send.send_error(format!("Unable to watch directory {:?}, launcher may be out of sync with files!", path)).await;
            return;
        }
        self.watching.insert(path.into(), target);
    }
    
    pub async fn remove_instance(&mut self, id: InstanceID) {
        if let Some(instance) = self.instances.get(id.index) {
            if instance.id == id {
                let instance = self.instances.remove(id.index);
                self.send.send(MessageToFrontend::InstanceRemoved { id }).await;
                self.send.send_info(format!("Instance '{}' removed", instance.name)).await;
            }
        }
    }
    
    pub async fn load_instance_from_path(&mut self, path: &Path, mut show_errors: bool, show_success: bool) -> bool {
        let instance = Instance::load_from_folder(&path).await;
        let Ok(mut instance) = instance else {
            if let Some(existing) = self.instance_by_path.get(path) {
                if let Some(existing_instance) = self.instances.get(existing.index) {
                    if existing_instance.id == *existing {
                        let instance = self.instances.remove(existing.index);
                        self.send.send(MessageToFrontend::InstanceRemoved { id: instance.id}).await;
                        show_errors = true;
                    }
                }
            }
            
            if show_errors {
                let error = instance.unwrap_err();
                self.send.send_error(format!("Unable to load instance from {:?}:\n{}", &path, &error)).await;
                eprintln!("Error loading instance: {:?}", &error);
            }
            
            return false;
        };
        
        if let Some(existing) = self.instance_by_path.get(path) {
            if let Some(existing_instance) = self.instances.get_mut(existing.index) {
                if existing_instance.id == *existing {
                    existing_instance.copy_basic_attributes_from(instance);
                    
                    let _ = self.send.send(existing_instance.create_modify_message()).await;
                    
                    if show_success {
                        self.send.send_info(format!("Instance '{}' updated", existing_instance.name)).await;
                    }
                    
                    return true;
                }
            }
        }
        
        let vacant = self.instances.vacant_entry();
        let instance_id = InstanceID {
            index: vacant.key(),
            generation: self.instances_generation,
        };
        self.instances_generation = self.instances_generation.wrapping_add(1);
        
        if show_success {
            self.send.send_success(format!("Instance '{}' created", instance.name)).await;
        }
        let message = MessageToFrontend::InstanceAdded {
            id: instance_id,
            name: instance.name.clone(),
            version: instance.version.clone(),
            loader: instance.loader,
            worlds_state: Arc::clone(&instance.worlds_state),
            servers_state: Arc::clone(&instance.servers_state),
        };
        instance.id = instance_id;
        vacant.insert(instance);
        let _ = self.send.send(message).await;
        
        self.instance_by_path.insert(path.to_owned(), instance_id);
        
        self.watch_filesystem(path, WatchTarget::InstanceDir { id: instance_id }).await;
        return true;
    }
    
    async fn handle(mut self) {
        let mut interval = tokio::time::interval(Duration::from_millis(1000));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        tokio::pin!(interval);
        
        loop {
            tokio::select! {
                message = self.recv.recv() => {
                    if let Some(message) = message {
                        self.handle_message(message).await;
                    } else {
                        eprintln!("Backend receiver has shut down");
                        break;
                    }
                },
                instance_change = self.watcher_rx.recv() => {
                    if let Some(instance_change) = instance_change {
                        self.handle_filesystem(instance_change).await;
                    } else {
                        eprintln!("Backend filesystem has shut down");
                        break;
                    }
                },
                _ = interval.tick() => {
                    self.handle_tick().await;
                }
            }
        }
    }
    
    async fn handle_tick(&mut self) {
        for (_, instance) in &mut self.instances {
            if let Some(child) = &mut instance.child {
                if !matches!(child.try_wait(), Ok(None)) {
                    instance.child = None;
                    self.send.send(instance.create_modify_message()).await;
                }
            }
            if let Some(summaries) = instance.finish_loading_worlds().await {
                self.send.send(MessageToFrontend::InstanceWorldsUpdated {
                    id: instance.id,
                    worlds: Arc::clone(&summaries)
                }).await;
                
                for summary in summaries.iter() {
                    if self.watcher.watch(&summary.level_path, notify::RecursiveMode::NonRecursive).is_ok() {
                        self.watching.insert(summary.level_path.clone(), WatchTarget::InstanceLevelDir {
                            id: instance.id,
                        });
                    }
                }
            }
            if let Some(summaries) = instance.finish_loading_servers().await {
                self.send.send(MessageToFrontend::InstanceServersUpdated {
                    id: instance.id,
                    servers: Arc::clone(&summaries)
                }).await;
            }
        }
    }
}
