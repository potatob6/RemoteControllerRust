use std::{process::{Child, Stdio, Command}, error::Error, thread::JoinHandle};
use crate::network_connector::Process;

pub struct ProcessRunning {
    pub cmd: Child,
    pub starting_args: Vec<String>,
    pub terminated_flag: u8,
    pub handlers: Vec<(JoinHandle<()>, JoinHandle<()>)>,
}

impl ProcessRunning {
    pub fn new_args(cmd: String, input_args: Vec<String>) -> Result<Self, Box<dyn Error>> {
        let mut args: Vec<String> = Vec::new();
        args.push(String::from(&cmd));
        for arg in input_args {
            args.push(arg);
        }

        let command = Command::new(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn();

        match command {
            Ok(c) => {
                Ok(Self {
                    cmd: c,
                    starting_args: args,
                    terminated_flag: 0,
                    handlers: vec![],
                })
            },
            Err(e) => { Err(Box::new(e)) }
        }

        
    }
    pub fn new(cmd: String) -> Result<Self, Box<dyn Error>> {
        Self::new_args(cmd, Vec::new())
    }
}

impl Process for ProcessRunning {
    fn terminate(&mut self) {
        self.terminated_flag = 1;
    }
}