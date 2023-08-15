pub mod cmd_handler;
pub mod cmd;
pub mod network_connector;
pub mod test;
pub mod command_parser;

fn main() {
    let network = network_connector::NetworkConnector::new(String::from("127.0.0.1:5050"));
    let result = network.join();

    println!("{:?}", result);
}