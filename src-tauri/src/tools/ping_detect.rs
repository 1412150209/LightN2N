use std::thread::sleep;
use std::time::Duration;

use log::debug;

use crate::tools::execute_command;

static RULE_NAME: &'static str = "LightN2N_Allow_Ping";

#[tauri::command]
pub fn ping_firewall_rule_check() -> Result<bool, String> {
    sleep(Duration::from_secs(1));
    match execute_command(
        "netsh",
        vec![
            "advfirewall",
            "firewall",
            "show",
            "rule",
            format!("name={}", RULE_NAME).as_str(),
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
pub fn ping_firewall_rule_add() -> Result<(), String> {
    match execute_command(
        "netsh",
        vec![
            "advfirewall",
            "firewall",
            "add",
            "rule",
            format!("name={}", RULE_NAME).as_str(),
            "protocol=icmpv4:8,any",
            "dir=in",
            "action=allow",
        ],
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

// #[tauri::command]
// pub fn ping_firewall_rule_rm() -> Result<(), String> {
//     match execute_command(
//         "netsh",
//         vec![
//             "advfirewall",
//             "firewall",
//             "delete",
//             "rule",
//             format!("name={}", RULE_NAME).as_str(),
//         ],
//     ) {
//         Ok(output_str) => {
//             debug!("{}", output_str);
//             if output_str.contains("No rules match the specified criteria")
//                 || output_str.contains("没有与指定标准相匹配的规则")
//                 || output_str.contains("确定")
//             {
//                 Ok(())
//             } else {
//                 Err(output_str)
//             }
//         }
//         Err(e) => Err(e.to_string()),
//     }
// }
