use std::sync::Arc;

use bridge::{instance::InstanceStatus, message::MessageToBackend, modal_action::ProgressTracker};
use schema::version::{LaunchArgument, LaunchArgumentValue};

use crate::{instance::{InstanceInfo, StartLoadResult}, launch::ArgumentExpansionKey, log_reader, metadata::manager::{AssetsIndexMetadata, MinecraftVersionManifestMetadata, MinecraftVersionMetadata, MojangJavaRuntimeComponentMetadata, MojangJavaRuntimesMetadata}, BackendState, WatchTarget};

impl BackendState {
    pub async fn handle_message(&mut self, message: MessageToBackend) {
        match message {
            MessageToBackend::LoadVersionManifest { reload } => {
                if reload {
                    self.meta.force_reload(&MinecraftVersionManifestMetadata).await;
                } else {
                    self.meta.load(&MinecraftVersionManifestMetadata).await;
                }
            },
            MessageToBackend::RequestLoadWorlds { id } => {
                if let Some(instance) = self.instances.get_mut(id.index) {
                    if instance.id == id {
                        if instance.start_load_worlds() == StartLoadResult::Initial {
                            let saves = instance.saves_path.clone();
                            
                            if self.watcher.watch(&saves, notify::RecursiveMode::NonRecursive).is_ok() {
                                self.watching.insert(saves.into(), WatchTarget::InstanceSavesDir {
                                    id: instance.id,
                                });
                            }
                        }
                    }
                }
            },
            MessageToBackend::RequestLoadServers { id } => {
                if let Some(instance) = self.instances.get_mut(id.index) {
                    if instance.id == id {
                        if instance.start_load_servers() == StartLoadResult::Initial {
                            let server_dat = instance.server_dat_path.clone();
                            
                            if self.watcher.watch(&server_dat, notify::RecursiveMode::NonRecursive).is_ok() {
                                self.watching.insert(server_dat.into(), WatchTarget::ServersDat {
                                    id: instance.id,
                                });
                            }
                        }
                    }
                }
            },
            MessageToBackend::CreateInstance { name, version, loader } => {
                if !crate::is_single_component_path(&*name) {
                    self.send.send_warning(format!("Unable to create instance, name must not be a path: {}", name)).await;
                    return;
                }
                if !sanitize_filename::is_sanitized_with_options(&*name, sanitize_filename::OptionsForCheck { windows: true, ..Default::default() }) {
                    self.send.send_warning(format!("Unable to create instance, name is invalid: {}", name)).await;
                    return;
                }
                if self.instances.iter().find(|(_, i)| i.name == name).is_some() {
                    self.send.send_warning(format!("Unable to create instance, name is already used")).await;
                    return;
                }
                
                let instance_dir = self.directories.instances_dir.join(name.as_str());
                
                let _ = tokio::fs::create_dir_all(&instance_dir).await;
                
                self.watch_filesystem(&self.directories.instances_dir.clone(), WatchTarget::InstancesDir).await;
                
                let instance_info = InstanceInfo {
                    minecraft_version: version.clone(),
                    loader,
                };
                
                let info_path = instance_dir.join("info_v1.json");
                tokio::fs::write(info_path, serde_json::to_string_pretty(&instance_info).unwrap()).await.unwrap();
            },
            MessageToBackend::KillInstance { id } => {
                if let Some(instance) = self.instances.get_mut(id.index) {
                    if instance.id == id {
                        if let Some(mut child) = instance.child.take() {
                            let result = child.kill();
                            if result.is_err() {
                                self.send.send_error("Failed to kill instance").await;
                                eprintln!("Failed to kill instance: {:?}", result.unwrap_err());
                            }
                            
                            self.send.send(instance.create_modify_message()).await;
                        } else {
                            self.send.send_error("Can't kill instance, instance wasn't running").await;
                        }
                        return;
                    }
                }
                
                self.send.send_error("Can't kill instance, unknown id").await;
            }
            MessageToBackend::StartInstance { id, quick_play, modal_action } => {
                if let Some(instance) = self.instances.get_mut(id.index) {
                    if instance.id == id {
                        if instance.child.is_some() {
                            self.send.send_warning("Can't launch instance, already running").await;
                            modal_action.set_error_message("Can't launch instance, already running".into());
                            modal_action.set_finished();
                            return;
                        }
                        
                        self.send.send(instance.create_modify_message_with_status(InstanceStatus::Launching)).await;
                        
                        let launch_tracker = ProgressTracker::new(Arc::from("Launching"), self.send.clone());
                        modal_action.trackers.push(launch_tracker.clone());
                        
                        let result = self.launcher.launch(&self.http_client, instance, quick_play, &launch_tracker, &modal_action).await;
                        
                        let is_err = result.is_err();
                        match result {
                            Ok(mut child) => {
                                if let Some(stdout) = child.stdout.take() {
                                    log_reader::start_game_output(stdout, self.send.clone());
                                }
                                instance.child = Some(child);
                            },
                            Err(ref err) => {
                                modal_action.set_error_message(format!("{}", &err).into());
                            },
                        }
                        
                        launch_tracker.set_finished(is_err);
                        launch_tracker.notify().await;
                        modal_action.set_finished();
                        
                        self.send.send(instance.create_modify_message()).await;
                        
                        return;
                    }
                }
                
                self.send.send_error("Can't launch instance, unknown id").await;
                modal_action.set_error_message("Can't launch instance, unknown id".into());
                modal_action.set_finished();
            },
            MessageToBackend::DownloadAllMetadata => {
                self.download_all_metadata().await;
            }
        }
    }
    
    pub async fn download_all_metadata(&self) {
        let Ok(versions) = self.meta.fetch(&MinecraftVersionManifestMetadata).await else {
            panic!("Unable to get Minecraft version manifest");
        };

        for link in &versions.versions {
            let Ok(version_info) = self.meta.fetch(&MinecraftVersionMetadata(link)).await else {
                panic!("Unable to get load version: {:?}", link.id);
            };
            
            let asset_index = format!("{}", version_info.assets);
            
            let Ok(_) = self.meta.fetch(&AssetsIndexMetadata {
                url: version_info.asset_index.url,
                cache: self.directories.assets_index_dir.join(format!("{}.json", &asset_index)).into(),
                hash: version_info.asset_index.sha1,
            }).await else {
                todo!("Can't get assets index {:?}", version_info.asset_index.url);
            };
            
            if let Some(arguments) = &version_info.arguments {
                for argument in arguments.game.iter() {
                    let value = match argument {
                        LaunchArgument::Single(launch_argument_value) => &launch_argument_value,
                        LaunchArgument::Ruled(launch_argument_ruled) => &launch_argument_ruled.value,
                    };
                    match value {
                        LaunchArgumentValue::Single(shared_string) => {
                            check_argument_expansions(shared_string.as_str());
                        },
                        LaunchArgumentValue::Multiple(shared_strings) => {
                            for shared_string in shared_strings.iter() {
                                check_argument_expansions(shared_string.as_str());
                            }
                        },
                    }
                }
            } else if let Some(legacy_arguments) = &version_info.minecraft_arguments {
                for argument in legacy_arguments.split_ascii_whitespace() {
                    check_argument_expansions(argument);
                }
            }
        }

        let Ok(runtimes) = self.meta.fetch(&MojangJavaRuntimesMetadata).await else {
            panic!("Unable to get java runtimes manifest");
        };

        for (platform_name, platform) in &runtimes.platforms {
            for (jre_component, components) in &platform.components {
                if components.is_empty() {
                    continue;
                }

                let runtime_component_dir = self.directories.runtime_base_dir.join(jre_component).join(platform_name.as_str());
                let _ = std::fs::create_dir_all(&runtime_component_dir);
                let Ok(runtime_component_dir) = runtime_component_dir.canonicalize() else {
                    panic!("Unable to create runtime component dir");
                };

                for runtime_component in components {
                    let Ok(manifest) = self.meta.fetch(&MojangJavaRuntimeComponentMetadata {
                        url: runtime_component.manifest.url,
                        cache: runtime_component_dir.join("manifest.json").into(),
                        hash: runtime_component.manifest.sha1,
                    }).await else {
                        panic!("Unable to get java runtime component manifest");
                    };

                    let keys: &[Arc<std::path::Path>] = &[
                        std::path::Path::new("bin/java").into(),
                        std::path::Path::new("bin/javaw.exe").into(),
                        std::path::Path::new("jre.bundle/Contents/Home/bin/java").into(),
                        std::path::Path::new("MinecraftJava.exe").into(),
                    ];

                    let mut known_executable_path = false;
                    for key in keys {
                        if manifest.files.contains_key(key) {
                            known_executable_path = true;
                            break;
                        }
                    }

                    if !known_executable_path {
                        eprintln!("Warning: {}/{} doesn't contain known java executable", jre_component, platform_name);
                    }
                }
            }
        }

        println!("Done downloading all metadata");
    }
}

fn check_argument_expansions(argument: &str) {
    let mut dollar_last = false;
    for (i, character) in argument.char_indices() {
        if character == '$' {
            dollar_last = true;
        } else if dollar_last && character == '{' {
            let remaining = &argument[i..];
            if let Some(end) = remaining.find('}') {
                let to_expand = &argument[i+1..i+end];
                if ArgumentExpansionKey::from_str(to_expand).is_none() {
                    eprintln!("Unsupported argument: {:?}", to_expand);
                }
            }
        } else {
            dollar_last = false;
        }
    }
}
