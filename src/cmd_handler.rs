use std::{sync::{mpsc::{self, RecvError, TryRecvError}, RwLock, Arc}, thread, io::{Write, BufReader, BufRead}};

use crate::{cmd::ProcessRunning, command_parser::HttpLikeData};
use crate::network_connector::{RequestString, ResponseString};

pub fn new_cmd_io_handler(cmd: Arc<RwLock<ProcessRunning>>) 
    -> (std::sync::mpsc::Sender<RequestString>, std::sync::mpsc::Receiver<ResponseString>) {

    let (process_input, p_recv) = mpsc::channel::<RequestString>();
    let (p_send, process_output) = mpsc::channel::<ResponseString>();

    let a1 = cmd.clone();
    let a1_read = cmd.clone();
    let a2_read = cmd.clone();
    let a3 = cmd.clone();

    let input_thread = thread::spawn(move || {
        let mut posses = cmd.write().unwrap();
        let recv = p_recv;
        let mut childin = posses.cmd.stdin.take().unwrap();
        drop(posses);

        childin.write_all("chcp 65001\r\n".as_bytes()).expect("Unable To Change To UTF8");
        loop {
            {
                let a1_read_r = a1_read.read();
                if let Err(_) = a1_read_r {
                    return;
                }
                let a1_read_r = a1_read_r.unwrap();
                if a1_read_r.terminated_flag == 1 {
                    drop(a1_read_r);
                    break;
                }
            }
            let o = recv.recv();
            match o {
                Ok(msg) => {

                    match msg {
                        RequestString::Request(msg) => {
                            childin.write_all(&msg.payload).expect("Write child error");
                            // dbg!(msg);
                        },
                        RequestString::Terminate(msg) => {
                            let mut lg = a1_read.write().unwrap();
                            (*lg).terminated_flag = 1;
                        }
                    }
                },
                Err(e) => {
                    dbg!("Encounter while recv from channel");
                    return;
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

            {
                let a2_read = a2_read.read();
                if let Err(_) = a2_read {
                    return;
                }
                let a2_read = a2_read.unwrap();
                if a2_read.terminated_flag == 1 {
                    drop(a2_read);
                    break;
                }
            }

            let mut buf = vec![];
            let _ = bufreader.read_until(b'\n', &mut buf);

            let reply = HttpLikeData::new()
                .header("Type", "Reply")
                .payload(&buf);

            let resp = ResponseString::Response(reply);
            let o = send.send(resp);
            // dbg!(String::from_utf8_lossy(&buf).to_string());
            match o {
                Err(e) => {
                    dbg!("Encounter error while send to channel");
                    return;
                },
                _ => { }
            }
        }
    });

    let mut a3_write = a3.write().unwrap();
    a3_write.handlers.push((input_thread, output_thread));

    (process_input, process_output)
}