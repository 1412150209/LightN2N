use std::fmt::Display;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;

use stunclient::StunClient;
use tauri::AppHandle;
use thiserror::Error;

use crate::config::LocalConfig;

#[derive(Debug)]
pub enum NatType {
    OpenInternet,
    FullCone,
    RestrictedCone,
    PortRestrictedCone,
    Symmetric,
}

#[derive(Debug, Error)]
pub enum NatError {
    #[error("Failed to resolve STUN server address")]
    StunServerResolutionError,
    #[error("Failed to bind to local address")]
    LocalBindError,
    #[error("UDP socket is blocked")]
    UdpBlocked,
    #[error("Symmetric UDP firewall detected")]
    SymmetricUdpFirewall,
    #[error("Unknown error occurred")]
    Unknown,
}

impl Display for NatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl NatType {
    fn perform_test_with_retries(
        udp: &UdpSocket,
        stun_addr: SocketAddr,
        retries: usize,
    ) -> Result<Option<SocketAddr>, NatError> {
        let client = StunClient::new(stun_addr);
        for _ in 0..retries {
            match client.query_external_address(udp) {
                Ok(addr) => return Ok(Some(addr)),
                Err(_) => continue,
            }
        }
        Ok(None)
    }

    pub fn detect(stun_server1: &str, stun_server2: &str) -> Result<Self, NatError> {
        let local_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
        let stun_addr = stun_server1
            .to_socket_addrs()
            .map_err(|_| NatError::StunServerResolutionError)?
            .find(|x| x.is_ipv4())
            .ok_or(NatError::StunServerResolutionError)?;

        let udp = UdpSocket::bind(local_addr).map_err(|_| NatError::LocalBindError)?;

        // Set a timeout for the UDP socket
        udp.set_read_timeout(Some(Duration::from_secs(2))).unwrap();

        // Test I: Initial STUN request
        let external_addr_1 = match Self::perform_test_with_retries(&udp, stun_addr, 3)? {
            Some(addr) => addr,
            None => return Err(NatError::UdpBlocked),
        };

        // Check if external address matches local address
        if external_addr_1.ip() == udp.local_addr().unwrap().ip() {
            return Ok(Self::OpenInternet);
        }

        // Test II: STUN request to a different STUN server
        let another_stun_addr = stun_server2
            .to_socket_addrs()
            .map_err(|_| NatError::StunServerResolutionError)?
            .find(|x| x.is_ipv4())
            .ok_or(NatError::StunServerResolutionError)?;

        let external_addr_2 = match Self::perform_test_with_retries(&udp, another_stun_addr, 3)? {
            Some(addr) => addr,
            None => return Err(NatError::SymmetricUdpFirewall),
        };

        if external_addr_1.ip() != external_addr_2.ip() {
            return Ok(Self::Symmetric);
        }

        // Test III: Rebind to another port and perform STUN request to the same server
        let new_udp = UdpSocket::bind(local_addr).map_err(|_| NatError::LocalBindError)?;
        new_udp
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        let external_addr_3 = match Self::perform_test_with_retries(&new_udp, stun_addr, 3)? {
            Some(addr) => addr,
            None => return Err(NatError::Unknown),
        };

        if external_addr_1 == external_addr_3 {
            return Ok(Self::FullCone);
        }

        // Check if port is the same
        return if external_addr_1.port() == external_addr_3.port() {
            Ok(Self::RestrictedCone)
        } else {
            Ok(Self::PortRestrictedCone)
        };
    }
}

#[tauri::command]
pub async fn nat_detect(app_handle: AppHandle) -> Result<String, String> {
    let config = LocalConfig::get_config(&app_handle);
    match NatType::detect(config.nat_detect[0].as_str(), config.nat_detect[1].as_str()) {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(e.to_string()),
    }
}
