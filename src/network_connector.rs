use std::{collections::HashMap, sync::{mpsc::{Sender, Receiver}, Arc, RwLock}, net::TcpStream, thread::{JoinHandle, self}, time::Duration, io::{BufReader, BufRead, ErrorKind}};

pub trait Process {
    fn terminate(&mut self);
}

pub enum ResponseString {
    Response(String, Option<i32>),
    Terminate(Option<String>, Option<i32>),
}

pub enum ResponseByte<const BUFSIZE: usize> {
    Response([u8; BUFSIZE], Option<i32>),
    Terminate(Option<[u8; BUFSIZE]>, Option<i32>),
}

pub enum RequestString {
    Request(String, Option<i32>),
    Terminate(Option<i32>),
}

pub enum RequestByte<const BUFSIZE: usize> {
    Terminate(Option<i32>),
    Request([u8; BUFSIZE], Option<i32>)
}

pub struct NetworkConnector<const BUFSIZE: usize> {
    pub ip: String,
    pub port: String,
    pub socket: TcpStream,

    pub processors_str: HashMap<usize, (Sender<RequestString>, Receiver<ResponseString>, Arc<RwLock<dyn Process>>)>,
    pub processors_byte: HashMap<usize, (Sender<RequestByte<BUFSIZE>>, Receiver<ResponseByte<BUFSIZE>>, Arc<RwLock<dyn Process>>)>,
}

// Input format: <seq number>[ <command> <data>]*

impl<const BUFSIZE: usize> NetworkConnector<BUFSIZE> {
    
    // Accepted IPv4 address
    fn new(host: String) -> JoinHandle<Result<u8, std::io::Error>> {
            thread::spawn( move || {
            let splits: Vec<&str> = host.split(':').collect();
            if splits.len() < 2 {
                panic!("Host Wrong.");
            }

            let socket = TcpStream::connect(&host[..]);

            match socket {
                Err(o) => {
                    return Err(o);
                },
                Ok(socket) => {
                    let mut myself = Self { 
                        ip: String::from(splits[0]), 
                        port: String::from(splits[1]), 
                        socket,
                        processors_str: HashMap::new(),
                        processors_byte: HashMap::new(),
                    };
                    myself.socket.set_read_timeout(Some(Duration::from_millis(10))).unwrap();

                    let mut recv_buf: Box<Vec<u8>> = Box::new(vec![]);
                    let mut buf_reader = BufReader::new(myself.socket);
    
                    loop {
                        // Starting read msg queue
                        let g = buf_reader.read_until(b'\\', &mut *recv_buf);
                        match &g {
                            Err(g) if g.kind() == ErrorKind::ConnectionReset => {
                                dbg!("Connection Reset");
                                break;
                            },
                            Err(g) if g.kind() == ErrorKind::TimedOut => {
                                dbg!("Encounter time out");
                            }
                            Ok(size) if *size == 0 => {
                                dbg!("Read the 0 size");
                                break;
                            }
                            _ => { }
                        }

                        if *recv_buf.last().unwrap() == b'\\' {
                            (*recv_buf).pop();
                            // Starting to sender to processes
                            
                            // Parsing input stream
                            // match command_parser::multi_command_parser(&s[..]) {
                            //     Some(inputcmd) => {
                            //         todo!("According command to execute");
                            //     },
                            //     None => {

                            //     }
                            // }
                        }
                    
                    // Starting receive msg queue
                    }

                    todo!("reset the tcpstream");
                }
            }
        })
    }
}