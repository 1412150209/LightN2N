use std::env::current_dir;
use std::fs::File;
use std::io::Write;

use log::error;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_store::{StoreCollection, with_store};

use crate::tools::{ExternalFilePosition, ProgramError};
use crate::tools::n2n_client::N2NClientConfig;

pub(crate) const REMOTE_CONFIG: &str = "远程配置文件";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalConfig {
    pub n2n_config: N2NClientConfig,
    pub nat_detect: Vec<String>,
    pub miniserve_port: u32,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            n2n_config: N2NClientConfig::default(),
            nat_detect: vec![
                "stun.nextcloud.com:3478".to_string(),
                "stun.miwifi.com:3478".to_string(),
            ],
            miniserve_port: 8090,
        }
    }
}

impl LocalConfig {
    /// 从store中取出config
    pub fn get_config(app_handle: &AppHandle) -> Self {
        let mut config = Self::default();
        let position = match current_dir() {
            Ok(position) => position.join(ExternalFilePosition::Config.to_string()),
            Err(e) => {
                error!(
                    "{}:{}",
                    line!(),
                    ProgramError::GetCurrentDirError(e.to_string())
                );
                return config;
            }
        };
        let stores = app_handle.state::<StoreCollection<Wry>>();
        if let Err(e) = with_store(app_handle.clone(), stores, position.clone(), |store| {
            match store.get("config") {
                None => {}
                Some(v) => config = serde_json::from_value(v.clone()).unwrap(),
            }
            Ok(())
        }) {
            error!(
                "{}:{}",
                line!(),
                ProgramError::ConfigGetError(e.to_string())
            );
        }
        config
    }

    /// 从网络下载默认配置覆盖本地配置
    pub fn download_config() -> Result<(), ProgramError> {
        match reqwest::blocking::get(REMOTE_CONFIG) {
            Ok(res) => {
                if res.status().is_success() {
                    match File::create(ExternalFilePosition::Config.to_string())
                        .map_err(|e| ProgramError::FileRWError(e.to_string()))
                    {
                        Ok(mut file) => {
                            match res.bytes() {
                                Ok(b) => {
                                    match file.write_all(b.as_ref()) {
                                        Ok(_) => {
                                            // 写入本地
                                            if let Err(e) = file.flush() {
                                                Err(ProgramError::ConfigGetError(e.to_string()))
                                            } else {
                                                Ok(())
                                            }
                                        }
                                        Err(e) => {
                                            Err(ProgramError::ConfigGetError(e.to_string()))
                                        }
                                    }
                                }
                                Err(e) => {
                                    Err(ProgramError::ConfigGetError(e.to_string()))
                                }
                            }
                        }
                        Err(e) => {
                            Err(e)
                        }
                    }
                } else {
                    Err(ProgramError::ConfigGetError(String::from("下载远程配置文件失败")))
                }
            }
            Err(e) => {
                Err(ProgramError::ConfigGetError(e.to_string()))
            }
        }
    }
}
