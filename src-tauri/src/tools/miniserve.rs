use std::any::Any;
use std::sync::{Arc, RwLock};

use log::{debug, error};
use tauri::AppHandle;

use crate::CHILDS;
use crate::config::LocalConfig;
use crate::tools::{child_drop, child_status, ChildProcess, execute_command, ExternalBinaryProgram, ExternalFilePosition, ProgramError};

pub struct MiniServe {
    program: ExternalBinaryProgram,
}

impl MiniServe {
    pub const NAME: &'static str = "miniserve";
    pub const FIRE_WALL_NAME: &'static str = "LightN2N_Allow_MiniServe";

    pub fn new(program_path: String, file_path: String, port: u32) -> Result<Self, ProgramError> {
        let current_dir =
            std::env::current_dir().map_err(|e| ProgramError::GetCurrentDirError(e.to_string()))?;

        let program = ExternalBinaryProgram::new(
            Self::NAME,
            current_dir.join(program_path),
            vec![
                file_path,
                "-p".to_string(),
                port.to_string(),
                // 显示隐藏文件
                "-H".to_string(),
                // 允许打包为tar
                "-r".to_string(),
                // 先显示文件夹
                "-D".to_string(),
                // 隐藏页面footer
                "-F".to_string(),
            ],
        )?;
        Ok(Self {
            program
        })
    }
}

impl ChildProcess for MiniServe {
    fn get_process(&mut self) -> &mut ExternalBinaryProgram {
        &mut self.program
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[tauri::command]
pub fn miniserve_start(app_handle: AppHandle, path: String) -> Result<(), String> {
    // 如果需要启动
    if !child_status(MiniServe::NAME) {
        let config = LocalConfig::get_config(&app_handle);
        match MiniServe::new(
            ExternalFilePosition::MiniServe.to_string(),
            path,
            config.miniserve_port,
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
                            map.insert(MiniServe::NAME, Arc::new(RwLock::new(client)));
                            Ok(())
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
        Ok(())
    }
}

#[tauri::command]
pub fn miniserve_stop() -> Result<bool, String> {
    child_drop(MiniServe::NAME)
}

#[tauri::command]
pub fn miniserve_firewall_check() -> Result<bool, String> {
    match execute_command(
        "netsh",
        vec![
            "advfirewall",
            "firewall",
            "show",
            "rule",
            format!("name={}", MiniServe::FIRE_WALL_NAME).as_str(),
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
pub fn miniserve_firewall_add() -> Result<(), String> {
    match std::env::current_dir().map_err(|e| ProgramError::GetCurrentDirError(e.to_string())) {
        Ok(current) => {
            match current
                .join(ExternalFilePosition::MiniServe.to_string())
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
                            format!("name={}", MiniServe::FIRE_WALL_NAME).as_str(),
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