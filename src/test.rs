#[cfg(test)]
pub mod test {
    use std::{net::{TcpStream, TcpListener}, time::Duration, io::{Write, Read}, thread};

    use crate::command_parser;

    #[test]
    fn command_parser_test() {
        let cmd = "Action:new\nSeq:2\n\n".as_bytes();
        dbg!(command_parser::multi_command_parser(cmd));
    }

    #[test]
    fn escape_decode_test() {
        let buf = "98wu80%3a%0a%5c%25%24%23%22".as_bytes();
        let m = String::from_utf8_lossy(&command_parser::escape_decode(buf)).to_string();
        dbg!(m);
    }

    #[test]
    fn t3() {
        loop {
            let m = Some(20);
            match m {
                Some(k) if k == 30 => {
                    dbg!(k);
                    break;
                }
                Some(k) if k == 20 => continue,
                _ => { }
            }
        }

    }

    #[test]
    fn t1() {
        let mut tcp = TcpStream::connect("127.0.0.1:6642").unwrap();

        thread::sleep(Duration::from_secs(5));
        let mut k = Box::new(Vec::with_capacity(4 * 1024 * 1024));
        unsafe { k.set_len(4 * 1024 * 1024) };
        let mut line = String::new();

        for r in 0..4 * 1024 * 1024 {
            (*k)[r] = b'2';
        }

        loop {
            std::io::stdin().read_line(& mut line).unwrap();
            let result = tcp.write(&k);
            match result {
                Ok(0) => { dbg!("Ok(0)"); },
                Ok(size) => { dbg!("Writed ", size); },
                Err(e) => { dbg!(e); },
            }
        }
    }

    #[test]
    fn t2() {
        let tcp = TcpListener::bind("0.0.0.0:6642").unwrap();
        for k in tcp.incoming() {
            let mut line = [0; 4 * 1024];
            match k {
                Ok(mut s) => {
                    s.set_read_timeout(Some(Duration::from_millis(300))).unwrap();

                    loop {
                        let r = s.read(&mut line);
                    
                        if let Ok(size) = &r {
                            if *size == 0 {
                                break;
                            }
                        }

                        std::io::stdout().write(b"[").unwrap();
                        for r in line {
                            std::io::stdout().write(&[r, b',']).unwrap();
                        }
                        std::io::stdout().write(b"]\n").unwrap();
                    }
                },
                _ => { }
            }
            
        }
    }
}