use std::net::IpAddr;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use log::{debug, error};
use ping::ping;

pub fn ping_average(
    address: IpAddr,
    timeout_duration: Duration,
    count: usize,
) -> Result<Duration, Box<dyn std::error::Error>> {
    let mut total_duration = Duration::new(0, 0);
    let mut success_count = 0;

    for i in 0..count {
        let (tx, rx) = mpsc::channel();
        let start = Instant::now();

        thread::spawn(move || {
            let result = ping(address, Some(timeout_duration), None, None, None, None);
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout_duration) {
            Ok(Ok(())) => {
                let duration = start.elapsed();
                total_duration += duration;
                success_count += 1;
                debug!("Ping {}: {:?}", i + 1, duration);
            }
            Ok(Err(e)) => error!("Error receiving reply: {}", e),
            Err(_) => error!("Ping {} timed out", i + 1),
        }

        thread::sleep(Duration::from_secs(1)); // 每次ping之间等待1秒
    }

    if success_count > 0 {
        Ok(total_duration / success_count as u32)
    } else {
        Err("无成功ping".into())
    }
}

#[tauri::command]
pub async fn ping_method(host: String) -> Result<u128, String> {
    let ip = IpAddr::from_str(host.as_str()).unwrap();
    return match ping_average(ip, Duration::from_secs(3), 3) {
        Ok(s) => Ok(s.as_millis()),
        Err(e) => Err(e.to_string()),
    };
}
