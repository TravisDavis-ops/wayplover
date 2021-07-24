use super::Config;
use super::Worker;
use std::io;
use std::sync::mpsc;
use std::thread;
use termion::{event::Key, input::TermRead};

#[derive(Debug)]
pub enum InputStatus {
    Input(Key),
}
pub enum InputControl {
    Shutdown,
}

pub struct InputWorker {
    tx: mpsc::Sender<InputControl>,
    rx: mpsc::Receiver<InputStatus>,
    handler: thread::JoinHandle<()>,
}

impl Worker<InputControl, InputStatus> for InputWorker {
    fn start(config: Config) -> Self {
        use std::thread::Builder;
        let ((tx, thread_rx), (thread_tx, rx)) = (mpsc::channel(), mpsc::channel());
        let handler = {
            let _config = config.clone();
            Builder::new()
                .name("WindowInput".to_string())
                .spawn(move || loop {
                    let stdin = io::stdin();
                    if let Ok(e) = thread_rx.try_recv() {
                        use InputControl::*;
                        match e {
                            Shutdown => {
                                break;
                            }
                        }
                    }

                    for key in stdin.keys() {
                        if let Ok(key) = key {
                            if let Err(_err) = thread_tx.send(InputStatus::Input(key)) {
                                continue;
                            }
                        }
                    }
                })
                .unwrap()
        };
        Self { rx, tx, handler }
    }
    fn recv(&self) -> Option<InputStatus> {
        self.rx.try_recv().ok()
    }
    fn send(&self, e: InputControl) {
        self.tx.send(e).unwrap()
    }
    fn shutdown(&self) {
        self.send(InputControl::Shutdown);
    }
}
