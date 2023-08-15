use std::{process::{Child, Stdio, Command}, error::Error, thread::JoinHandle};
use crate::network_connector::Process;

pub struct ProcessRunning {
    pub cmd: Child,
    pub starting_args: String,
    pub terminated_flag: u8,
    pub handlers: Vec<(JoinHandle<()>, JoinHandle<()>)>,
}

impl ProcessRunning {
    pub fn new_str_args(cmd: String, input_args: String) -> Result<Self, Box<dyn Error>> {
        let command = Command::new(&cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg(&input_args)
            .spawn();

        // dbg!("Started process running");
        // dbg!(&command);

        match command {
            Ok(c) => {
                Ok(Self {
                    cmd: c,
                    starting_args: cmd + &input_args,
                    terminated_flag: 0,
                    handlers: vec![],
                })
            },
            Err(e) => { Err(Box::new(e)) }
        }
    }
    pub fn new_args(cmd: String, input_args: Vec<String>) -> Result<Self, Box<dyn Error>> {
        let mut args: String = String::new();
        for k in input_args {
            args += &k;
        }
        ProcessRunning::new_str_args(cmd, args)
    }
    pub fn new(cmd: String) -> Result<Self, Box<dyn Error>> {
        ProcessRunning::new_str_args(cmd, String::new())
    }
}

impl Process for ProcessRunning {
    fn terminate(&mut self) {
        self.terminated_flag = 1;
    }
}