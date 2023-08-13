pub mod cmd_handler;
pub mod cmd;
pub mod network_connector;
pub mod test;
pub mod command_parser;
fn main() {

    let a = String::from_utf8(b"fufk%%\\".to_vec()).unwrap();
    let result = a.replace("%", "%25")
        .replace("\\", "%ff");

    dbg!(&result);

    let result2 = result.replace("%ff", "\\")
        .replace("%25", "%");

    dbg!(result2);


    // let pr = Arc::new(RwLock::new(ProcessRunning::new(String::from("cmd.exe")).expect("Creating error")));
    // let (send, _) = new_cmd_io_handler(pr);

    // loop {
    //     let mut line = String::new();
    //     std::io::stdin().read_line(&mut line).expect("Input Error");

    //     send.send(line);
    // }
}