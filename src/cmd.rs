use std::{process::{Child, Stdio, Command}, error::Error};

pub struct ProcessRunning {
    pub cmd: Child,
    pub starting_args: Vec<String>,
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
                })
            },
            Err(e) => { Err(Box::new(e)) }
        }

        
    }
    pub fn new(cmd: String) -> Result<Self, Box<dyn Error>> {
        Self::new_args(cmd, Vec::new())
    }
}