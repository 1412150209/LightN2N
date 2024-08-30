use std::any::Any;
use std::fmt::{Display, Formatter};
use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;

use chardet::detect;
use encoding_rs::Encoding;
use log::{debug, error, info};
use thiserror::Error;

use crate::CHILDS;

pub mod adapter_check;
pub mod miniserve;
pub mod n2n_client;
pub mod n2n_controller;
pub mod nat_detect;
pub mod ping;
pub mod ping_detect;
pub mod win_ip_broadcast;

/// 外部文件位置
pub enum ExternalFilePosition {
    N2NClient,
    WinIPBroadcast,
    Config,
    MiniServe,
}

impl Display for ExternalFilePosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let prefix: &str = "client";
        match self {
            ExternalFilePosition::N2NClient => {
                write!(f, "{}\\x64\\edge.exe", prefix)
            }
            ExternalFilePosition::WinIPBroadcast => {
                write!(f, "{}\\x64\\WinIPBroadcast.exe", prefix)
            }
            ExternalFilePosition::Config => {
                write!(f, "{}\\config.json", prefix)
            }
            ExternalFilePosition::MiniServe => {
                write!(f, "{}\\x64\\miniserve.exe", prefix)
            }
        }
    }
}

/// 外部二进制程序
pub struct ExternalBinaryProgram {
    name: String,
    process: Option<Child>,
    path: PathBuf,
    args: Vec<String>,
}

impl Drop for ExternalBinaryProgram {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

impl ExternalBinaryProgram {
    pub fn new(name: &str, path: PathBuf, command_args: Vec<String>) -> Result<Self, ProgramError> {
        Ok(Self {
            name: name.to_string(),
            process: None,
            path,
            args: command_args,
        })
    }

    pub fn stop(&mut self) -> Result<(), ProgramError> {
        match self.process {
            None => Ok(()),
            Some(ref mut p) => p
                .kill()
                .map_err(|e| ProgramError::ChildProcessError(e.to_string())),
        }
    }

    pub fn status(&mut self) -> bool {
        match self.process {
            None => false,
            Some(ref mut p) => match p.try_wait() {
                Ok(o) => match o {
                    None => true,
                    Some(_) => false,
                },
                Err(e) => {
                    error!("{}", ProgramError::ChildProcessError(e.to_string()));
                    false
                }
            },
        }
    }

    pub fn start(&mut self) -> Result<(), ProgramError> {
        // 如果程序正在运行
        if self.status() {
            Err(ProgramError::CreateTwice(self.name.to_string()))
        } else {
            debug!("启动子进程：{:?}\n参数:{:?}", self.path, self.args);
            match Command::new(self.path.clone())
                .args(self.args.clone())
                .stdout(Stdio::piped())
                .creation_flags(0x08000000)
                .spawn()
            {
                Ok(mut child) => {
                    // 创建线程输出日志
                    let stdout = child.stdout.take().unwrap();
                    let name = self.name.clone();
                    thread::spawn(move || {
                        BufReader::new(stdout)
                            .lines()
                            .filter_map(|line| line.ok())
                            .for_each(|line| info!("{}:{}", name, line))
                    });
                    self.process = Some(child);
                    Ok(())
                }
                Err(e) => Err(ProgramError::ChildProcessError(e.to_string())),
            }
        }
    }
}

/// 隐藏执行命令，获取输出
pub fn execute_command(command: &str, args: Vec<&str>) -> Result<String, ProgramError> {
    let output = match Command::new(command)
        .args(args)
        .creation_flags(0x08000000)
        .output()
    {
        Ok(s) => s,
        Err(e) => {
            return Err(ProgramError::CommandRunningError(e.to_string()));
        }
    };
    // 检测编码格式
    let detected = detect(&output.stdout);
    let encoding = Encoding::for_label(detected.0.as_ref()).unwrap_or(encoding_rs::UTF_8);
    // 将输出转换为字符串
    let (decoded_str, _, _) = encoding.decode(&output.stdout);
    Ok(decoded_str.into_owned())
}

/// 子进程
pub trait ChildProcess: Sync + Send {
    fn get_process(&mut self) -> &mut ExternalBinaryProgram;

    fn as_any(&mut self) -> &mut dyn Any;
}

/// 查询子进程运行状况
pub fn child_status(name: &str) -> bool {
    match CHILDS.lock() {
        Ok(map) => {
            match map.get(name) {
                // 未启动
                None => {
                    false;
                }
                // 已启动
                Some(s) => {
                    // 是否退出
                    match s.write() {
                        Ok(mut s) => {
                            return s.get_process().status();
                        }
                        Err(e) => {
                            let error = ProgramError::ChildProcessError(e.to_string()).to_string();
                            error!("{}:{}", line!(), error);
                        }
                    }
                }
            };
        }
        Err(e) => {
            let error = ProgramError::ChildProcessError(e.to_string()).to_string();
            error!("{}:{}", line!(), error);
        }
    }
    false
}

pub fn child_drop(name: &str) -> Result<bool, String> {
    match CHILDS.lock() {
        Ok(mut map) => match map.remove(name) {
            None => Ok(true),
            Some(e) => match e.write() {
                Ok(s) => {
                    drop(s);
                    Ok(true)
                }
                Err(e) => {
                    let error = ProgramError::ChildProcessError(e.to_string()).to_string();
                    error!("{}:{}", line!(), error);
                    Err(error)
                }
            },
        },
        Err(e) => {
            let error = ProgramError::ChildProcessError(e.to_string()).to_string();
            error!("{}:{}", line!(), error);
            Err(error)
        }
    }
}

#[derive(Debug, Error)]
pub enum ProgramError {
    #[error("子进程错误:{0}")]
    ChildProcessError(String),
    #[error("未找到子进程")]
    ChildProcessNotFound,
    #[error("重复创建进程:{0}")]
    CreateTwice(String),
    #[error("获取当前路径错误:{0}")]
    GetCurrentDirError(String),
    #[error("Downcast错误")]
    DowncastError,
    #[error("转换错误")]
    TransferError,
    #[error("未能获取到参数:{0}")]
    ParameterGetError(String),
    #[error("获取标准输出错误")]
    StdoutGetError,
    #[error("命令执行失败:{0}")]
    CommandRunningError(String),
    #[error("获取配置文件失败:{0}")]
    ConfigGetError(String),
    #[error("文件读写失败:{0}")]
    FileRWError(String),
    #[error("网络错误:{0}")]
    NetworkError(String),
    #[error("防火墙错误")]
    FireWallError,
}
