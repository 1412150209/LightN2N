use std::os::windows::process::CommandExt;
use std::process::Command;

use chardet::detect;
use log::{error, warn};
use tauri_plugin_shell::process::Encoding;

use crate::tools::ProgramError;

const ADAPTER_NAME: &str = "TAP-Windows Adapter";

/// 检测是否安装虚拟网卡
fn check_adapter() -> Result<bool, ProgramError> {
    return match check_adapter_wmic() {
        Ok(s) => Ok(s),
        Err(e) => {
            warn!("{}", e);
            check_adapter_ipconfig()
        }
    };
}

fn check_adapter_wmic() -> Result<bool, ProgramError> {
    let output = Command::new("wmic")
        .creation_flags(0x08000000)
        .args(&["nic", "list", "brief"])
        .output()
        .map_err(|_| ProgramError::StdoutGetError)?;
    // 检测编码格式
    let detected = detect(&output.stdout);
    let encoding = Encoding::for_label(detected.0.as_ref()).unwrap_or(encoding_rs::UTF_8);
    // 将输出转换为字符串
    let (decoded_str, _, _) = encoding.decode(&output.stdout);
    let output_str = decoded_str.into_owned();
    return Ok(output_str.contains(ADAPTER_NAME));
}

fn check_adapter_ipconfig() -> Result<bool, ProgramError> {
    // 执行ipconfig命令
    let output = Command::new("ipconfig")
        .creation_flags(0x08000000)
        .args(&["/all"])
        .output()
        .map_err(|_| ProgramError::StdoutGetError)?;
    // 检测编码格式
    let detected = detect(&output.stdout);
    let encoding = Encoding::for_label(detected.0.as_ref()).unwrap_or(encoding_rs::UTF_8);
    // 将输出转换为字符串
    let (decoded_str, _, _) = encoding.decode(&output.stdout);
    let output_str = decoded_str.into_owned();
    return Ok(output_str.contains(ADAPTER_NAME));
}

#[tauri::command]
pub fn n2n_check_adapter() -> bool {
    match check_adapter() {
        Ok(b) => {
            return b;
        }
        Err(e) => {
            error!("{}", e);
            false
        }
    }
}
