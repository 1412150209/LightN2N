use std::any::Any;
use std::sync::{Arc, RwLock};

use log::error;

use crate::tools::{
    child_drop, child_status, ChildProcess, ExternalBinaryProgram, ExternalFilePosition,
    ProgramError,
};
use crate::CHILDS;

pub struct WinIPBroadcast {
    process: ExternalBinaryProgram,
}

impl WinIPBroadcast {
    pub const NAME: &'static str = "WinIPBroadcast";

    pub fn new(program_path: String) -> Result<Self, ProgramError> {
        let current_dir =
            std::env::current_dir().map_err(|e| ProgramError::GetCurrentDirError(e.to_string()))?;
        let program = ExternalBinaryProgram::new(
            Self::NAME,
            current_dir.join(program_path),
            vec!["run".to_string()],
        )?;
        Ok(Self { process: program })
    }
}

impl ChildProcess for WinIPBroadcast {
    fn get_process(&mut self) -> &mut ExternalBinaryProgram {
        &mut self.process
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[tauri::command]
pub fn win_ip_broadcast_start() -> Result<bool, String> {
    // 如果子进程未正常运行
    if !child_status(WinIPBroadcast::NAME) {
        return match WinIPBroadcast::new(ExternalFilePosition::WinIPBroadcast.to_string()) {
            Ok(mut p) => {
                match p.get_process().start() {
                    Ok(_) => {
                        // 运行后保存
                        match CHILDS.lock() {
                            Ok(mut map) => {
                                map.insert(WinIPBroadcast::NAME, Arc::new(RwLock::new(p)));
                                Ok(true)
                            }
                            Err(e) => {
                                // 防止多开
                                let error =
                                    ProgramError::ChildProcessError(e.to_string()).to_string();
                                error!("{}", error);
                                drop(p);
                                Err(error)
                            }
                        }
                    }
                    Err(e) => {
                        let error = e.to_string();
                        error!("{}", error);
                        Err(error)
                    }
                }
            }
            Err(e) => {
                let error = e.to_string();
                error!("{}", error);
                Err(error)
            }
        };
    }
    Ok(true)
}

#[tauri::command]
pub fn win_ip_broadcast_stop() -> Result<bool, String> {
    child_drop(WinIPBroadcast::NAME)
}

#[tauri::command]
pub fn win_ip_broadcast_status() -> Result<bool, String> {
    Ok(child_status(WinIPBroadcast::NAME))
}
