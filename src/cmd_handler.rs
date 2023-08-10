use std::{sync::{mpsc, RwLock, Arc}, thread::{JoinHandle, self}, io::{Write, BufReader, BufRead}};

use crate::cmd::ProcessRunning;

pub fn new_cmd_io_handler(cmd: Arc<RwLock<ProcessRunning>>) 
    -> (std::sync::mpsc::Sender<String>, std::sync::mpsc::Receiver<String>, JoinHandle<()>, JoinHandle<()>) {

    let (process_input, p_recv) = mpsc::channel::<String>();
    let (p_send, process_output) = mpsc::channel::<String>();

    let a1 = cmd.clone();

    let input_thread = thread::spawn(move || {
        let mut posses = cmd.write().unwrap();
        let recv = p_recv;
        let mut childin = posses.cmd.stdin.take().unwrap();
        drop(posses);

        childin.write_all("chcp 65001\r\n".as_bytes()).expect("Unable To Change To UTF8");

        loop {
            let o = recv.recv();
            match o {
                Ok(msg) => {
                    let k = childin.write_all(msg.as_bytes());
                    dbg!(&k);
                    dbg!(msg);
                },
                Err(e) => { 
                    dbg!(e);
                }
            }
        }

    });

    let output_thread = thread::spawn(move || {
        let mut posses = a1.write().unwrap();
        let childout = posses.cmd.stdout.take().unwrap();

        drop(posses);

        let mut bufreader = BufReader::new(childout);
        let send = p_send;
        loop {
            let mut buf = vec![];
            let _ = bufreader.read_until(b'\n', &mut buf);
            let o = send.send(String::from_utf8_lossy(&buf).to_string());
            dbg!(String::from_utf8_lossy(&buf).to_string());
            match o {
                Err(e) => {
                    dbg!(e);
                },
                _ => { }
            }
        }
    });

    (process_input, process_output, input_thread, output_thread)
}