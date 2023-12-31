#[cfg(test)]
pub mod test {
    use std::{net::{TcpStream, TcpListener}, time::Duration, io::{Write, Read, BufReader, BufRead, ErrorKind}, thread, collections::HashMap, sync::mpsc, error::Error};

    use crate::{command_parser::{self, HttpLikeData}, network_connector};

    #[test]
    fn client_test() {
        let network = network_connector::NetworkConnector::new(String::from("127.0.0.1:5050"));

        println!("Waiting");
        let result = network.join();
    }

    #[test]
    fn server_test() {
        let tcp = TcpListener::bind("0.0.0.0:5050").unwrap();
        for k in tcp.incoming() {
            let mut k = k.unwrap();
            let data = HttpLikeData::new()
                .header("Action", "New Command")
                .header("Program", "cmd");

            k.write_all(&data.to_network_stream()[..]).unwrap();
            println!("Writed");
            
            let mut k = BufReader::new(k);
            let mut buf = vec![];
            loop {
                k.read_until(b'\\', &mut buf).unwrap();
                let data = HttpLikeData::multi_command_parse(&buf[..]);
                dbg!(&data);

                buf.clear();
            }
        }
    }

    #[test]
    fn channel_broken_test() {
        let (s, r) = mpsc::channel();
        thread::spawn(move || {
            s.send(String::from("fuck"));
        });

        let k = thread::spawn(move || {
            thread::sleep(Duration::from_millis(1000));
            loop {
                let recv = r.recv_timeout(Duration::from_secs(1));
                if let Err(t) = recv {
                    dbg!(t);
                }
            }
        });

        k.join();
    }

    #[test]
    fn wrong_network_stream_parse_test() {
        let m = b"((U$(H@(D\n@)$Uuwui\narhiw\n\narwairw\\";
        let p = HttpLikeData::multi_command_parse(m).unwrap();
        let r = p.to_network_stream();
        let g = HttpLikeData::multi_command_parse(&r[..]).unwrap();
        
        assert_eq!(g, p);
    }

    #[test]
    fn httplikedata_constructor_test() {
        let m = HttpLikeData::new()
            .header("Action", "Drop")
            .header("ACK_SEQ", "1")
            .header("File Name", "E:\\share\\goto")
            .payload(b"Fuck your self::*&(@)%@(*(");

        let k = m.to_network_stream();
        let m1 = HttpLikeData::multi_command_parse(&k[..]);
        assert_eq!(m, m1.unwrap());
    }

    #[test]
    fn encoder_escape_test() {
        let m = "oh289%::\\\n".as_bytes();
        let s = String::from_utf8_lossy(&command_parser::escape_encode(&m[..])).to_string();
        println!("{s}");
    }

    #[test]
    fn command_parser_test() {
        let cmd = "Type:Reply\nAck Seq:0\n\nE%3a%5cRustProjects%5chello_world>chcp 65001\r%0a\\".as_bytes();
        dbg!(HttpLikeData::multi_command_parse(cmd));
    }

    // :(3a) \n(0a) \(5c) %(25)
    #[test]
    fn incoming_data_decode_test() {
        let cmd = "Action:new\nSeq:2\nFile Name:\"E%3a%5cshare_lock%5cshare\"\n\n27184yh291j4291n%5c%25".as_bytes();
        // dbg!(String::from_utf8_lossy(&HttpLikeData::multi_command_parse(cmd).unwrap().payload[..]).to_string());
    }

    #[test]
    fn escape_decode_test() {
        let buf = "98wu80%3a%0a%5c%25%24%23%22".as_bytes();
        let m = String::from_utf8_lossy(&command_parser::escape_decode(buf)).to_string();
        // dbg!(m);
    }

    #[test]
    fn t3() {
        loop {
            let m = Some(20);
            match m {
                Some(k) if k == 30 => {
                    // dbg!(k);
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
                Ok(0) => { 
                    // dbg!("Ok(0)");
                },
                Ok(size) => {
                    //  dbg!("Writed ", size); 
                },
                Err(e) => {
                    //  dbg!(e); 
                },
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