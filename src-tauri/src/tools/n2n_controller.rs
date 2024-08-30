use std::collections::HashMap;
use std::net::UdpSocket;
use std::time::Duration;

use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tools::ProgramError;

#[derive(Serialize, Deserialize, Debug)]
struct JsonResponse {
    _tag: String,
    _type: String,
    #[serde(flatten)]
    data: HashMap<String, Value>,
}

pub struct Controller {
    address: String,
    port: u16,
    tag: u32,
    key: Option<String>,
    debug: bool,
    sock: UdpSocket,
}

#[derive(Serialize, Clone, Debug)]
pub struct Member {
    pub address: String,
    pub name: String,
    pub mode: String,
}

impl PartialEq for Member {
    fn eq(&self, other: &Self) -> bool {
        if self.address.eq(&other.address) {
            if self.mode.eq(&other.mode) {
                if self.name.eq(&other.name) {
                    return true;
                }
            }
        }
        return false;
    }
}

impl Controller {
    pub fn new(port: u16) -> Self {
        let sock = UdpSocket::bind("0.0.0.0:0").expect("Couldn't bind to address");
        sock.set_read_timeout(Some(Duration::from_secs(3)))
            .expect("Couldn't set read timeout");

        Controller {
            address: "127.0.0.1".to_string(),
            port,
            tag: 0,
            key: None,
            debug: false,
            sock,
        }
    }

    fn next_tag(&mut self) -> String {
        let tagstr = self.tag.to_string();
        self.tag = (self.tag + 1) % 1000;
        tagstr
    }

    fn cmdstr(&mut self, msgtype: &str, cmdline: &str) -> (String, String) {
        let tagstr = self.next_tag();
        let mut options = vec![tagstr.clone()];
        if let Some(ref key) = self.key {
            options.push("1".to_string());
            options.push(key.clone());
        }
        let optionsstr = options.join(":");
        (tagstr, format!("{} {} {}", msgtype, optionsstr, cmdline))
    }

    fn rx(&self, tagstr: &str) -> Result<Vec<HashMap<String, Value>>, String> {
        let mut buffer = [0; 1024];
        let mut result = Vec::new();

        loop {
            let (size, _) = self
                .sock
                .recv_from(&mut buffer)
                .map_err(|e| e.to_string())?;
            let data: JsonResponse =
                serde_json::from_slice(&buffer[..size]).map_err(|e| e.to_string())?;

            if data._tag != tagstr {
                continue;
            }

            match data._type.as_str() {
                // 数据类型
                "error" => {
                    return Err(data
                        .data
                        .get("error")
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string());
                }
                "end" => {
                    return Ok(result);
                }
                "row" => {
                    if self.debug {
                        debug!("{:?}", data.data);
                    }
                    result.push(data.data);
                }
                "begin" => {}
                "subscribed" => {
                    if self.debug {
                        debug!("Subscribed")
                    }
                }
                "unsubscribed" => {
                    if self.debug {
                        debug!("Unsubscribed")
                    }
                }
                "event" => {
                    result.push(data.data);
                }
                _ => {
                    return Err(format!("Unknown data type {}", data._type));
                }
            }
        }
    }

    fn call(
        &mut self,
        msgtype: &str,
        cmdline: &str,
    ) -> Result<Vec<HashMap<String, Value>>, String> {
        let (tagstr, msgstr) = self.cmdstr(msgtype, cmdline);
        self.sock
            .send_to(msgstr.as_bytes(), format!("{}:{}", self.address, self.port))
            .map_err(|e| e.to_string())?;
        self.rx(&tagstr)
    }

    fn read(&mut self, cmdline: &str) -> Result<Vec<HashMap<String, Value>>, String> {
        self.call("r", cmdline)
    }

    fn write(&mut self, cmdline: &str) -> Result<Vec<HashMap<String, Value>>, String> {
        self.call("w", cmdline)
    }

    /// 获取虚拟ip
    pub fn get_vip(&mut self) -> Result<String, ProgramError> {
        match self.read("info") {
            Ok(response) => {
                for row in response {
                    match row.get("ip4addr") {
                        None => {
                            error!("{}", ProgramError::ParameterGetError("虚拟ip".to_string()))
                        }
                        Some(s) => {
                            return Ok(s.as_str().ok_or(ProgramError::TransferError)?.to_string());
                        }
                    };
                }
            }
            Err(e) => {
                error!("{}", e);
            }
        }
        return Ok(String::from("0.0.0.0"));
    }

    /// 通过管理端口关闭客户端
    pub fn close(&mut self) -> Result<Vec<HashMap<String, Value>>, String> {
        self.write("stop")
    }

    /// 检查客户端是否在线
    pub fn test(&mut self) -> bool {
        return match self.read("help") {
            Ok(_) => true,
            Err(_) => false,
        };
    }

    /// 查询当前所在组
    pub fn current_group(&mut self) -> Result<String, ProgramError> {
        match self.read("communities") {
            Ok(s) => {
                for row in s {
                    match row.get("community") {
                        None => {
                            error!("{}", ProgramError::ParameterGetError("当前组".to_string()));
                        }
                        Some(s) => {
                            return Ok(s.as_str().ok_or(ProgramError::TransferError)?.to_string());
                        }
                    }
                }
            }
            Err(e) => {
                error!("{}", e);
            }
        };
        return Ok(String::from("None"));
    }

    /// 查询当前组所有成员
    pub fn edges(&mut self) -> Vec<Member> {
        let mut members = Vec::<Member>::new();
        match self.read("edges") {
            Ok(s) => {
                for row in s {
                    members.push(Member {
                        address: match row.get("ip4addr") {
                            None => "None".to_string(),
                            Some(s) => {
                                if s.eq("") {
                                    "None".to_string()
                                } else {
                                    s.as_str()
                                        .ok_or(ProgramError::TransferError)
                                        .unwrap()
                                        .to_string()
                                }
                            }
                        },
                        name: match row.get("desc") {
                            None => "None".to_string(),
                            Some(s) => {
                                if s.eq("") {
                                    "None".to_string()
                                } else {
                                    s.as_str()
                                        .ok_or(ProgramError::TransferError)
                                        .unwrap()
                                        .to_string()
                                }
                            }
                        },
                        mode: match row.get("mode") {
                            None => "Unknown".to_string(),
                            Some(s) => {
                                if s.eq("") {
                                    "Unknown".to_string()
                                } else {
                                    s.as_str()
                                        .ok_or(ProgramError::TransferError)
                                        .unwrap()
                                        .to_string()
                                }
                            }
                        },
                    })
                }
            }
            Err(e) => {
                error!("{}", e)
            }
        }
        return members;
    }
}
