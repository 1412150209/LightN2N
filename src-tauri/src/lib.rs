use std::collections::HashMap;
use std::env::current_dir;
use std::sync::{Arc, Mutex, RwLock};

use flexi_logger::{Cleanup, Criterion, DeferredNow, FileSpec, Logger, Naming};
use flexi_logger::filter::{LogLineFilter, LogLineWriter};
use lazy_static::lazy_static;
use log::{error, Record, warn};
use tauri::{AppHandle, Manager, Wry};
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
use tauri_plugin_store::{StoreCollection, with_store};

use crate::config::LocalConfig;
use crate::tools::{child_drop, ChildProcess, ExternalFilePosition, ProgramError};
use crate::tools::adapter_check::n2n_check_adapter;
use crate::tools::miniserve::{miniserve_firewall_add, miniserve_firewall_check, miniserve_start, miniserve_stop};
use crate::tools::n2n_client::{
    n2n_client_start, n2n_client_stop, n2n_firewall_add, n2n_firewall_check, n2n_members,
    n2n_self_ip, n2n_status,
};
use crate::tools::nat_detect::nat_detect;
use crate::tools::ping::ping_method;
use crate::tools::ping_detect::{
    ping_firewall_rule_add, ping_firewall_rule_check,
};
use crate::tools::win_ip_broadcast::{
    win_ip_broadcast_start, win_ip_broadcast_status, win_ip_broadcast_stop,
};

mod config;
mod tools;

lazy_static! {
    static ref CHILDS: Mutex<HashMap<&'static str, Arc<RwLock<dyn ChildProcess>>>> =
        Mutex::new(HashMap::new());
}

// 无痛退出
fn safe_exit(app_handle: AppHandle) {
    let mut to_drop = Vec::new();
    match CHILDS.lock() {
        Ok(process) => {
            for p in process.keys() {
                to_drop.push(p.to_string())
            }
        }
        Err(_) => {}
    }
    for p in to_drop {
        let _ = child_drop(p.as_str());
    }
    app_handle.exit(0);
}

struct LogFilter;

impl LogLineFilter for LogFilter {
    fn write(
        &self,
        now: &mut DeferredNow,
        record: &Record,
        log_line_writer: &dyn LogLineWriter,
    ) -> std::io::Result<()> {
        if !record.args().to_string().contains("RedrawEventsCleared") {
            log_line_writer.write(now, record)?;
        }
        Ok(())
    }
}

pub fn run() {
    // 保存日志到本地文件
    Logger::try_with_str("info")
        .expect("Failed to log")
        .filter(Box::new(LogFilter))
        .log_to_file(
            FileSpec::default()
                .directory("logs")
                .basename("light_n2n")
                .suffix("log"),
        )
        .rotate(
            Criterion::Size(5000000000),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(3),
        )
        .start()
        .expect("Failed to log");

    // 检查本地是否有配置文件
    let position = match current_dir() {
        Ok(position) => position.join(ExternalFilePosition::Config.to_string()),
        Err(e) => {
            error!("{}:{}",line!(),ProgramError::GetCurrentDirError(e.to_string()));
            panic!();
        }
    };
    if !position.exists() {
        // 从服务器拉取配置
        warn!("{}",ProgramError::ConfigGetError(String::from("本地没有配置文件")));
        if let Err(e) = LocalConfig::download_config() {
            error!("{}:{}",line!(),ProgramError::ConfigGetError(e.to_string()));
            panic!();
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(webview_window) = app.get_webview_window("main") {
                let _ = webview_window.show();
                let _ = webview_window.set_focus();
            }
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        // 初始化
        .setup(move |app| {
            // 加载本地配置
            let app_handle = app.app_handle().clone();
            let collection = app.state::<StoreCollection<Wry>>();

            if let Err(e) = with_store(app_handle.clone(), collection, position.clone(), |store| {
                match store.load() {
                    Ok(_) => {
                        match store.get("config") {
                            None => {}
                            Some(v) => {
                                match serde_json::from_value::<LocalConfig>(v.clone()) {
                                    Ok(_) => {}
                                    Err(e) => {
                                        warn!("{}:{}", line!(), ProgramError::ConfigGetError(e.to_string()));
                                        // 配置不符合规则，重新拉取
                                        if let Err(e) = LocalConfig::download_config() {
                                            error!("{}:{}",line!(),ProgramError::ConfigGetError(e.to_string()));
                                            panic!();
                                        } else {
                                            store.load()?
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        Err(e)?
                    }
                }
                Ok(())
            }) {
                error!("{}:{}",line!(),ProgramError::ConfigGetError(e.to_string()));
            }
            // 托盘
            let exit = MenuItemBuilder::with_id("exit", "退出").build(app)?;
            let open = MenuItemBuilder::with_id("open", "显示主界面").build(app)?;
            let menu = MenuBuilder::new(app).items(&[&open, &exit]).build()?;
            match app.tray_by_id("main") {
                None => {}
                Some(tray) => {
                    tray.set_menu(Some(menu))?;
                    tray.on_menu_event(move |app, event| match event.id().as_ref() {
                        "open" => {
                            if let Some(webview_window) = app.get_webview_window("main") {
                                let _ = webview_window.show();
                                let _ = webview_window.set_focus();
                            }
                        }
                        "exit" => {
                            safe_exit(app_handle.clone());
                        }
                        _ => (),
                    });
                    tray.on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(webview_window) = app.get_webview_window("main") {
                                let _ = webview_window.show();
                                let _ = webview_window.set_focus();
                            }
                        }
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            n2n_client_start,
            n2n_client_stop,
            n2n_self_ip,
            n2n_status,
            n2n_members,
            win_ip_broadcast_stop,
            win_ip_broadcast_start,
            win_ip_broadcast_status,
            n2n_check_adapter,
            ping_method,
            nat_detect,
            ping_firewall_rule_check,
            ping_firewall_rule_add,
            n2n_firewall_check,
            n2n_firewall_add,
            miniserve_start,
            miniserve_stop,
            miniserve_firewall_add,
            miniserve_firewall_check
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
