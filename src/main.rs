use std::sync::{Arc, RwLock};

use cmd::ProcessRunning;
use cmd_handler::new_cmd_io_handler;
pub mod cmd_handler;
pub mod cmd;
pub mod network_connector;

fn main() {
    let pr = Arc::new(RwLock::new(ProcessRunning::new(String::from("cmd.exe")).expect("Creating error")));
    let (send, _) = new_cmd_io_handler(pr);

    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).expect("Input Error");

        send.send(line);
    }
}