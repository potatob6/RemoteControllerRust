use std::{collections::HashMap, sync::{mpsc::{Sender, Receiver}, Arc, RwLock}, net::TcpStream, thread::JoinHandle};

pub trait Process {
    fn terminate(&mut self);
}

pub struct ProcessorGlobal {
    pub sends: Vec<Sender<String>>,
    pub recvs: Vec<Receiver<String>>,
    pub entity: Arc<RwLock<dyn Process>>,
}

pub struct NetworkConnector {
    pub ip: String,
    pub port: String,
    pub socket: Arc<RwLock<TcpStream>>,

    pub pause_msg_send: Arc<RwLock<u8>>,
    pub pause_msg_recv: Arc<RwLock<u8>>,

    pub msg_handler: Option<JoinHandle<()>>,
    pub netword_handler: Option<JoinHandle<()>>,

    pub processors: HashMap<u32, ProcessorGlobal>,
}

impl NetworkConnector {
    
    // Accepted IPv4 address
    fn new(host: String) -> Self {
        let splits: Vec<&str> = host.split(':').collect();
        if splits.len() < 2 {
            panic!("Host Wrong.");
        }

        let socket = TcpStream::connect(&host[..]).expect("Connection Error");

        Self { 
            ip: String::from(splits[0]), 
            port: String::from(splits[1]), 
            socket: Arc::new(RwLock::from(socket)),
            pause_msg_send: Arc::new(RwLock::new(0)), 
            pause_msg_recv: Arc::new(RwLock::new(0)), 
            msg_handler: None, 
            netword_handler: None,
            processors: HashMap::new(), 
        }
    }

    fn create_msg_handler(&mut self) {
        
    }
}