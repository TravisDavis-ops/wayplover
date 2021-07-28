#[cfg(test)]
extern crate test_case;

use super::Config;
use log::{info, error};
use super::Worker;
use crate::steno::Chord;
use serial;
use std::io::Read;
use std::sync::mpsc;
use std::thread;

const BYTES_PER_STROKE: usize = 6;
const STENO_MAP: [&str; 42] = [
    "Fn", "#", "#", "#", "#", "#", "#", "S-", "S-", "T-", "K-", "P-", "W-", "H-", "R-", "A-", "O-",
    "*", "*", "res", "res", "pwr", "*", "*", "-E", "-U", "-F", "-R", "-P", "-B", "-L", "-G", "-T",
    "-S", "-D", "#", "#", "#", "#", "#", "#", "-Z",
];

pub enum DeviceStatus {
    Input(Chord),
    Disconnected,
}
#[derive(Debug)]
pub enum DeviceControl {
    Disable,
    Enable,
    Reconnect(&'static str),
    Disconnect,
    Shutdown,
}
pub struct SerialWorker {
    tx: mpsc::Sender<DeviceControl>,
    rx: mpsc::Receiver<DeviceStatus>,
    handler: thread::JoinHandle<()>,
}
impl Worker<DeviceControl, DeviceStatus> for SerialWorker {
    fn start(config: Config) -> Self {
        let ((tx, thread_rx), (thread_tx, rx)) = (mpsc::channel(), mpsc::channel());
        let handler = thread::Builder::new()
            .name("DeviceThread".to_string())
            .spawn(move || {
                let mut port = serial::open(&config.port).ok();
                loop {
                    if let Ok(e) = thread_rx.try_recv() {
                        info!("{:?}", e);
                        println!("{:?}", e);
                        use DeviceControl::*;
                        match e {
                            Disconnect => {
                                //let _  = port.take();
                                thread_tx.send(DeviceStatus::Disconnected);
                            }
                            Reconnect(path) => {
                                //port = serial::open(path).ok();
                            }
                            Shutdown => {
                            }
                            _ => {}
                        }
                    }
                    let mut byte_chord: [u8; BYTES_PER_STROKE] = [0; BYTES_PER_STROKE];
                    match port.as_mut() {
                        Some(p) => {
                            if let Err(e) = p.read_exact(&mut byte_chord){
                                
                            }
                        }
                        None => loop {

                            use std::time::Duration;
                            thread::sleep(Duration::from_millis(30));
                            use DeviceControl::*;
                            if let Ok(e) = thread_rx.try_recv() {
                                match e {
                                    Reconnect(path) => {
                                        port = serial::open(path).ok();
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    let mut temp = Vec::new();
                    for (i, b) in byte_chord.iter().enumerate() {
                        for j in 1..8 {
                            if b & (0x80 >> j) != 0 {
                                temp.push(STENO_MAP[i * 7 + j - 1]);
                            }
                        }
                    }
                    let chord = Chord::new(temp);
                    if !chord.is_empty() {
                        thread_tx.send(DeviceStatus::Input(chord)).unwrap();
                    }
                }
            })
            .unwrap();
        Self { tx, rx, handler }
    }

    fn send(&self, e: DeviceControl) {
        info!("[SerialEvent] {:?}", e);
        self.tx.send(e).expect("Send Error");
    }
    fn shutdown(&self) {
        self.send(DeviceControl::Shutdown);
    }

    fn recv(&self) -> Option<DeviceStatus> {
        if let Ok(status) = self.rx.try_recv() {
            Some(status)
        } else {
            None
        }
    }
}

