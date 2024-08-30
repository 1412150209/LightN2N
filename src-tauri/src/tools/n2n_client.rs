use std::any::Any;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;

use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;

use crate::CHILDS;
use crate::config::LocalConfig;
use crate::tools::{
    child_drop, child_status, ChildProcess, execute_command, ExternalBinaryProgram,
    ExternalFilePosition, ProgramError,
};
use crate::tools::n2n_controller::{Controller, Member};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct N2NClientConfig {
    pub identification: String,
    pub group: String,
    pub server: String,
    pub port: u16,
    pub member_server: String,
    pub control_port: u16,
}

impl Default for N2NClientConfig {
    fn default() -> Self {
        Self {
            identification: "Default".to_string(),
            group: "lers10".to_string(),
            server: "服务器地址".to_string(),
            port: 49898,
            member_server: "成员服务器地址".to_string(),
            control_port: 5644,
        }
    }
}

pub struct N2NClient {
    program: ExternalBinaryProgram,
    controller: Controller,
}

impl Drop for N2NClient {
    fn drop(&mut self) {
        let _ = self.controller.close();
        let _ = self.program.stop();
    }
}

impl N2NClient {
    pub const NAME: &'static str = "n2n_client";
    pub const FIRE_WALL_NAME: &'static str = "LightN2N_Allow_N2N";

    pub fn new(config: N2NClientConfig, program_path: String) -> Result<Self, ProgramError> {
        let current_dir =
            std::env::current_dir().map_err(|e| ProgramError::GetCurrentDirError(e.to_string()))?;

        let program = ExternalBinaryProgram::new(
            Self::NAME,
            current_dir.join(program_path),
            vec![
                "-c".to_string(),
                config.group.clone(),
                "-l".to_string(),
                format!("{}:{}", config.server, config.port),
                "-I".to_string(),
                config.identification.clone(),
                "-E".to_string(),
                "-p".to_string(),
                config.port.clone().to_string(),
            ],
        )?;
        let controller = Controller::new(config.control_port);
        Ok(Self {
            program,
            controller,
        })
    }
}

impl ChildProcess for N2NClient {
    fn get_process(&mut self) -> &mut ExternalBinaryProgram {
        &mut self.program
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[tauri::command]
pub fn n2n_client_start(app_handle: AppHandle) -> Result<bool, String> {
    // 如果需要启动
    if !child_status(N2NClient::NAME) {
        let config = LocalConfig::get_config(&app_handle);
        match N2NClient::new(
            config.n2n_config,
            ExternalFilePosition::N2NClient.to_string(),
        ) {
            Ok(mut client) => {
                if let Err(e) = client.get_process().start() {
                    let error = e.to_string();
                    error!("{}:{}", line!(), error);
                    Err(error)
                } else {
                    // 运行后保存
                    match CHILDS.lock() {
                        Ok(mut map) => {
                            map.insert(N2NClient::NAME, Arc::new(RwLock::new(client)));
                            Ok(true)
                        }
                        Err(e) => {
                            // 防止多开
                            let error = ProgramError::ChildProcessError(e.to_string()).to_string();
                            error!("{}:{}", line!(), error);
                            drop(client);
                            Err(error)
                        }
                    }
                }
            }
            Err(e) => {
                let error = e.to_string();
                error!("{}:{}", line!(), error);
                Err(error)
            }
        }
    } else {
        Ok(true)
    }
}

#[tauri::command]
pub fn n2n_client_stop() -> Result<bool, String> {
    child_drop(N2NClient::NAME)
}

#[tauri::command]
pub fn n2n_status() -> Result<bool, String> {
    if child_status(N2NClient::NAME) {
        match CHILDS.lock() {
            Ok(s) => match s.get(N2NClient::NAME) {
                None => Err(ProgramError::ChildProcessNotFound.to_string()),
                Some(p) => match p.write() {
                    Ok(mut s) => match s.as_any().downcast_mut::<N2NClient>() {
                        None => Err(ProgramError::DowncastError.to_string()),
                        Some(c) => {
                            for _ in 0..3 {
                                if c.controller.test() {
                                    return Ok(true);
                                }
                                sleep(Duration::from_millis(600));
                            }
                            Err(ProgramError::ChildProcessNotFound.to_string())
                        }
                    },
                    Err(e) => Err(ProgramError::ChildProcessError(e.to_string()).to_string()),
                },
            },
            Err(e) => Err(ProgramError::ChildProcessError(e.to_string()).to_string()),
        }
    } else {
        Ok(false)
    }
}

#[tauri::command]
pub fn n2n_self_ip() -> Result<String, String> {
    if child_status(N2NClient::NAME) {
        match CHILDS.lock() {
            Ok(s) => match s.get(N2NClient::NAME) {
                None => Err(ProgramError::ChildProcessNotFound.to_string()),
                Some(p) => match p.write() {
                    Ok(mut s) => match s.as_any().downcast_mut::<N2NClient>() {
                        None => Err(ProgramError::DowncastError.to_string()),
                        Some(c) => match c.controller.get_vip() {
                            Ok(s) => Ok(s),
                            Err(e) => Err(e.to_string()),
                        },
                    },
                    Err(e) => Err(ProgramError::ChildProcessError(e.to_string()).to_string()),
                },
            },
            Err(e) => Err(ProgramError::ChildProcessError(e.to_string()).to_string()),
        }
    } else {
        Err(ProgramError::ChildProcessNotFound.to_string())
    }
}

#[tauri::command]
pub fn n2n_members(app_handle: AppHandle) -> Result<Vec<Member>, String> {
    if child_status(N2NClient::NAME) {
        match CHILDS.lock() {
            Ok(s) => match s.get(N2NClient::NAME) {
                None => Err(ProgramError::ChildProcessNotFound.to_string()),
                Some(p) => match p.write() {
                    Ok(mut s) => match s.as_any().downcast_mut::<N2NClient>() {
                        None => Err(ProgramError::DowncastError.to_string()),
                        Some(c) => {
                            let config = LocalConfig::get_config(&app_handle);
                            let member_server = format!(
                                "{}/members/{}",
                                config.n2n_config.member_server, config.n2n_config.group
                            );
                            if c.controller.test() {
                                let mut temp = Vec::<Member>::new();
                                match reqwest::blocking::get(member_server) {
                                    Ok(response) => {
                                        if response.status().is_success() {
                                            match response.text() {
                                                Ok(res) => {
                                                    match serde_json::from_str::<Value>(&res) {
                                                        Ok(res) => {
                                                            match c.controller.get_vip() {
                                                                Ok(me) => {
                                                                    match res["status"].as_bool() {
                                                                        None => {
                                                                            Err(ProgramError::TransferError.to_string())
                                                                        }
                                                                        Some(s) => {
                                                                            if s {
                                                                                let Some(members) = res["members"].as_array() else { return Err(ProgramError::TransferError.to_string()); };
                                                                                for member in members {
                                                                                    let Some(ip) = member["ip4addr"].as_str() else { return Err(ProgramError::TransferError.to_string()); };
                                                                                    if ip.split("/").collect::<Vec<_>>()[0] != me {
                                                                                        temp.push(Member {
                                                                                            address: ip.to_string(),
                                                                                            name: member["desc"].as_str().unwrap_or("Default").to_string(),
                                                                                            mode: "None".to_string(),
                                                                                        })
                                                                                    }
                                                                                }
                                                                                for member in c.controller.edges() {
                                                                                    match temp.iter().position(|x| { x.address == member.address }) {
                                                                                        None => {}
                                                                                        Some(index) => {
                                                                                            temp[index].mode = member.mode
                                                                                        }
                                                                                    }
                                                                                }
                                                                                Ok(temp)
                                                                            } else {
                                                                                Err(ProgramError::TransferError.to_string())
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                Err(e) => Err(e.to_string()),
                                                            }
                                                        }
                                                        Err(e) => {
                                                            Err(ProgramError::ChildProcessError(
                                                                e.to_string(),
                                                            )
                                                                .to_string())
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    Err(ProgramError::NetworkError(e.to_string())
                                                        .to_string())
                                                }
                                            }
                                        } else {
                                            Err(ProgramError::NetworkError(
                                                response.status().to_string(),
                                            )
                                                .to_string())
                                        }
                                    }
                                    Err(e) => {
                                        Err(ProgramError::NetworkError(e.to_string()).to_string())
                                    }
                                }
                            } else {
                                Err(ProgramError::ChildProcessError(String::from(
                                    "Controller Test Failed",
                                ))
                                    .to_string())
                            }
                        }
                    },
                    Err(e) => Err(ProgramError::ChildProcessError(e.to_string()).to_string()),
                },
            },
            Err(e) => Err(ProgramError::ChildProcessError(e.to_string()).to_string()),
        }
    } else {
        Err(ProgramError::ChildProcessNotFound.to_string())
    }
}

#[tauri::command]
pub fn n2n_firewall_check() -> Result<bool, String> {
    match execute_command(
        "netsh",
        vec![
            "advfirewall",
            "firewall",
            "show",
            "rule",
            format!("name={}", N2NClient::FIRE_WALL_NAME).as_str(),
        ],
    ) {
        Ok(output_str) => {
            debug!("{}", output_str);
            if output_str.contains("No rules match the specified criteria")
                || output_str.contains("没有与指定标准相匹配的规则")
            {
                Ok(false)
            } else {
                Ok(true)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn n2n_firewall_add() -> Result<(), String> {
    match std::env::current_dir().map_err(|e| ProgramError::GetCurrentDirError(e.to_string())) {
        Ok(current) => {
            match current
                .join(ExternalFilePosition::N2NClient.to_string())
                .to_str()
            {
                None => Err(ProgramError::FireWallError.to_string()),
                Some(path) => {
                    match execute_command(
                        "netsh",
                        vec![
                            "advfirewall",
                            "firewall",
                            "add",
                            "rule",
                            format!("name={}", N2NClient::FIRE_WALL_NAME).as_str(),
                            "dir=in",
                            "action=allow",
                            format!("program={}", path).as_str(),
                            "enable=yes",
                        ],
                    ) {
                        Ok(output_str) => {
                            debug!("{}", output_str);
                            Ok(())
                        }
                        Err(e) => Err(e.to_string()),
                    }
                }
            }
        }
        Err(e) => Err(e.to_string()),
    }
}
