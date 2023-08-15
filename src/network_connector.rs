use std::{collections::HashMap, sync::{mpsc::{Sender, Receiver, RecvTimeoutError}, Arc, RwLock}, net::TcpStream, thread::{JoinHandle, self}, time::Duration, io::{BufReader, BufRead, ErrorKind, Write, BufWriter}, error::Error, cell::RefCell};

use crate::{command_parser::HttpLikeData, cmd::ProcessRunning, cmd_handler};

pub trait Process {
    fn terminate(&mut self);
}
#[derive(Debug)]
pub enum ResponseString {
    Response(HttpLikeData),
    Terminate(HttpLikeData),
}
#[derive(Debug)]
pub enum ResponseByte {
    Response(HttpLikeData),
    Terminate(HttpLikeData),
}
#[derive(Debug)]
pub enum RequestString {
    Request(HttpLikeData),
    Terminate(HttpLikeData),
}
#[derive(Debug)]
pub enum RequestByte {
    Request(HttpLikeData),
    Terminate(HttpLikeData),
}

pub struct NetworkConnector {
    pub ip: String,
    pub port: String,
    pub terminated: bool,
    pub next_index: usize,

    pub processors: HashMap<usize, (Sender<RequestString>, Receiver<ResponseString>, Arc<RwLock<dyn Process>>)>,
    // pub processors_byte: HashMap<usize, (Sender<RequestByte>, Receiver<ResponseByte>, Arc<RwLock<dyn Process>>)>,
    pub self_msg: Vec<HttpLikeData>,
}

impl Drop for NetworkConnector {
    fn drop(&mut self) {
        for k in &self.processors {
            {
                let mut locker = k.1.2.write().unwrap();
                locker.terminate();
            }
        }
    }
}

// Input format: <seq number>[ <command> <data>]*

impl NetworkConnector {
    
    // Accepted IPv4 address
    // Return: Err(1) = socket establish error
    //         Err(2) = Host ip or port error
    //         Err(3) = Encounter connection reset while read buf
    //         Err(4) = Encounter reading 0 size from read buf(close by peer)
    //         Err(5) = Encounter an unknown error
    //         Err(6) = Cloning socket stream fail
    //         Err(7) = Encounter an error while write out socket
    pub fn new(host: String) -> JoinHandle<Result<u8, u8>> {
            let network_handler = thread::spawn( move || {
            let splits: Vec<&str> = host.split(':').collect();
            if splits.len() < 2 {
                return Err(2);
            }

            let socket = TcpStream::connect(&host[..]);

            let mut result: Result<u8, u8> = Ok(0);
            match socket {
                Err(o) => {
                    result = Err(1);
                },
                Ok(mut socket) => {
                    let mut myself = Self { 
                        ip: String::from(splits[0]), 
                        port: String::from(splits[1]), 
                        terminated: false,
                        processors: HashMap::new(),
                        // processors_byte: HashMap::new(),
                        self_msg: vec![],
                        next_index: 0,
                    };

                    socket.set_read_timeout(Some(Duration::from_millis(10))).unwrap();

                    let mut recv_buf: Vec<u8> = vec![];
                    let socket_clone = socket.try_clone();
                    if let Err(_) = socket_clone {
                        return Err(6);
                    }
                    let mut buf_reader = BufReader::new(socket_clone.unwrap());
                    'outter: loop {
                        let mut need_drop = vec![];
                        // Starting read msg queue
                        let g = buf_reader.read_until(b'\\', &mut recv_buf);
                        match &g {
                            Err(g) if g.kind() == ErrorKind::ConnectionReset => {
                                dbg!("Connection Reset");
                                result = Err(3);
                                break;
                            },
                            Err(g) if g.kind() == ErrorKind::TimedOut => {
                                // dbg!("Encounter time out");
                            }
                            Ok(size) if *size == 0 => {
                                dbg!("Read the 0 size");
                                result = Err(4);
                                break;
                            }
                            _ => { }
                        }

                        if recv_buf.len() != 0 {
                            let data = HttpLikeData::multi_command_parse(&recv_buf[..]);
                            // dbg!(&data);
                            match data {
                                Some(val) => {
                                    let (myself1, err_stream) = NetworkConnector::do_with_incoming_data_stream(myself, val);
                                    myself = myself1;

                                    if let Some(val) = err_stream {
                                        need_drop.push(val);
                                    }
                                },
                                None => {
                                    // Drop this package
                                }
                            }
                        }

                        recv_buf.clear();
                        // Starting handle output of processes
                        {
                            // Starting self message send
                            let iter = myself.self_msg.iter();
                            for self_msg in iter {
                                let data = &self_msg.to_network_stream()[..];
                                let send_result = socket.write_all(data);
                                if let Err(_) = send_result {
                                    result = Err(7);
                                    break 'outter;
                                }
                                dbg!("Sended", self_msg, String::from_utf8_lossy(data).to_string());
                            }
                            myself.self_msg.clear();

                            let iter = myself.processors.iter();
                            for queue_str in iter {
                                let data = queue_str.1.1.recv_timeout(Duration::from_millis(100));
                                if let Err(g) = data {
                                    if g == RecvTimeoutError::Disconnected {
                                        println!("Read from channel disconnect: {:?}", g);
                                        {
                                            let locker = queue_str.1.2.write();
                                            locker.unwrap().terminate();
                                        }
                                        let data = HttpLikeData::new()
                                            .header("Status", "Terminate")
                                            .header("Index", &(*queue_str.0).to_string());
                                        need_drop.push(*queue_str.0);
                                        continue;
                                    }
                                }

                                let data = data.unwrap();
                                match data {
                                    ResponseString::Response(mut data) => {
                                        data = data.header("Ack Seq", &queue_str.0.to_string()[..]);
                                        let send_data = &data.to_network_stream()[..];
                                        if let Err(_) = socket.write_all(send_data) {
                                            result = Err(7);
                                            break 'outter;
                                        }
                                        dbg!("Sended", &data, String::from_utf8_lossy(send_data).to_string());
                                    },
                                    ResponseString::Terminate(data) => {

                                    }
                                }
                            }
                        }

                        for need_drops in need_drop {
                            (&mut myself.processors).remove(&need_drops);
                        }
                    }
                    result = Err(5)
                }
            };
            result
        });
        network_handler
    }


    // Incoming data headers: 
    // Action(New Command):
    //                      Program: The executable file of process running.
    //        [Alternative] Args: The args of program.
    // Action(Input Command):
    // Action(New File Input):
    // Action(Input File):
    // Action(Terminate):


    // Output data headers:
    // Status(Error):
    //                      Action: The reply of action.
    //                      Error Message: The error message of error.
    //        [Alternative] Program: Indicates the program of error encountered.
    // Status(Success):
    //                      Action: The reply of action.
    //        [Alternative] Program: Indicates the program of success encountered.
    //        [Alternative] Index: The index of reply process.
    // Status(Terminate):

    pub fn do_with_incoming_data_stream(mut self: Self, data: HttpLikeData) -> (Self, Option<usize>) {
        let action = data.headers.get("Action");
        if let Some(val) = action {
            if val == "New Command" {
                // Do something ...
                let program = data.headers.get("Program");
                match program {
                    None => {
                        let error_msg = HttpLikeData::new()
                            .header("Status", "Error")
                            .header("Error Message", "Unknown starting program.");

                        self.self_msg.push(error_msg);
                    },
                    Some(program) => {
                        let args = data.headers.get("Args");

                        if let Some(args) = args {
                            let args = String::from(args);
                            let pr = ProcessRunning::new_str_args(String::from(program), args);
                            if let Err(_) = pr {
                                let error_msg = HttpLikeData::new()
                                    .header("Status", "Error")
                                    .header("Error Message", "Unable to start program.");
                                dbg!(&error_msg);
                                self.self_msg.push(error_msg);
                                return (self, None);
                            }

                            let pr = Arc::new(RwLock::from(pr.unwrap()));
                            let (i, o) = cmd_handler::new_cmd_io_handler(pr.clone());
                            self.processors.insert(self.next_index, (i, o, pr));

                            let data = HttpLikeData::new()
                                .header("Status", "Success")
                                .header("Prorgram", &String::from(program))
                                .header("Index", &self.next_index.to_string());
                            self.self_msg.push(data);
                            self.next_index += 1;

                        } else {
                            let pr = ProcessRunning::new(String::from(program));
                            if let Err(_) = pr {
                                let error_msg = HttpLikeData::new()
                                    .header("Status", "Error")
                                    .header("Error Message", "Unable to start program.");
                                dbg!(&error_msg);
                                self.self_msg.push(error_msg);
                                return (self, None);
                            }

                            let pr = Arc::new(RwLock::from(pr.unwrap()));
                            let (i, o) = cmd_handler::new_cmd_io_handler(pr.clone());
                            self.processors.insert(self.next_index, (i, o, pr));
                            let data = HttpLikeData::new()
                                .header("Status", "Success")
                                .header("Prorgram", &String::from(program))
                                .header("Index", &self.next_index.to_string());
                            self.self_msg.push(data);
                            self.next_index += 1;
                        }
                    }
                }
                
            } else if val == "Input Command" {

            } else if val == "New File Input" {

            } else if val == "Input File" {

            } else if val == "Terminate" {

            }
        }
        (self, None)
    }
}