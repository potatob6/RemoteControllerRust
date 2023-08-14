use std::collections::HashMap;

#[derive(Debug)]
pub struct HttpLikeData {
    pub headers: HashMap<String, String>,
    pub payload: Vec<u8>,
}

const E1: &[u8; 3] = b"%3a";
const E2: &[u8; 3] = b"%0a";
const E3: &[u8; 3] = b"%5c";
const E4: &[u8; 3] = b"%25";

// Test Complete[√]
// Escape encode for :(3a) \n(0a) \(5c) %(25)
pub fn escape_encode(buf: &[u8]) -> Vec<u8> {
    let mut result = vec![];
    for l in buf {
        if *l == b':' {
            result.append(&mut E1.to_vec());
            continue;
        }

        if *l == b'\n' {
            result.append(&mut E2.to_vec());
            continue;
        }

        if *l == b'\\' {
            result.append(&mut E3.to_vec());
            continue;
        }

        if *l == b'%' {
            result.append(&mut E4.to_vec());
            continue;
        }

        result.push(*l);
    }
    result
}

const TABLE: [(&[u8; 3], u8); 4] = [(b"%3a", b':'), (b"%0a", b'\n'), (b"%5c", b'\\'), (b"%25", b'%')];
// Test Complete[√]
// Escape decode for :(3a) \n(0a) \(5c) %(25)
pub fn escape_decode(buf: &[u8]) -> Vec<u8> {
    let mut v = vec![];
    let mut decode_buf = None;

    for n in buf {
        if *n == b'%' {
            decode_buf = Some(vec![b'%']);
            continue;
        }
        
        decode_buf = match decode_buf {
            Some(mut v1) => {
                v1.push(*n);
                if v1.len() == 3 {
                    let mut finded = None;
                    for k in TABLE {
                        if k.0.to_vec() == v1 {
                            finded = Some(k.1);
                        }
                    }

                    match finded {
                        Some(v2) => {
                            v.push(v2);
                            None
                        },
                        None => {
                            // If read the escape code is not in the tablet, you can decide add it to result with origin sample
                            // example: 
                            //    read: %24 (not in the tablet), result: %24
                            // if not execute v.append:
                            //    read: %24 (not in the tablet), result: (nothing)
                            // v.append(&mut v1);
                            None
                        },
                    }
                } else {
                    Some(v1)
                }
            },
            None => {
                v.push(*n);
                None
            }
        };
    }
    v
}

impl HttpLikeData {
    // Test Complete[√]
    pub fn multi_command_parse<'a>(cmd: &'a [u8]) -> Option<HttpLikeData> {
        if cmd[cmd.len()-1] != b'\\' {
            return None;
        }
        let cmd = &cmd[..cmd.len() - 1];

        //Split by "\n\n"
        let mut counter = 0;
        let iters = cmd.split(|num| {
            if *num == b'\n' {
                counter += 1;
            } else {
                counter = 0;
            }
            if counter == 2 { return true; }
            false
        });
        
        let mut ref_headers = &[0u8; 0] as &[u8];
        let mut ref_payload = &[0u8; 0] as &[u8];

        let mut split_by_n_number = 0u32;

        for (i, k) in iters.enumerate() {
            if i == 0 {
                ref_headers = k;
            } else if i == 1 {
                ref_payload = k;
            }
            split_by_n_number += 1;
        }

        if split_by_n_number != 2 {
            return None;
        }

        let mut vec_headers = HashMap::new();
        ref_headers = &ref_headers[..ref_headers.len() - 1];
        for k in ref_headers.split(|num| *num == b'\n') {
            if k.len() != 0 {
                let mut iter1 = k.split(|num| *num == b':');

                let key = iter1.next();
                let value = iter1.next();

                match value {
                    Some(val) => {
                        let split_kv = (
                            String::from_utf8_lossy(&escape_encode(key.unwrap())[..]).to_string(),
                            String::from_utf8_lossy(&escape_encode(val)[..]).to_string()
                        );
                        vec_headers.insert(split_kv.0, split_kv.1);
                    },
                    None => {
                        let split_kv = (
                            String::from_utf8_lossy(&escape_encode(key.unwrap())[..]).to_string(),
                            String::from_utf8_lossy(b"").to_string()
                        );
                        vec_headers.insert(split_kv.0, split_kv.1);
                    }
                }
            }
        }

        Some(HttpLikeData { 
            headers: vec_headers,
            payload: escape_decode(ref_payload),
        })
        // Parse command and data

    }

    pub fn new() -> Self {
        Self { headers: HashMap::new(), payload: vec![] }
    }

    pub fn header(self, key: &str, value: &str) -> Self {
        // let key = String::from_utf8_lossy(&escape_encode(key.as_bytes())[..]).to_string();
        // let value = String::from_utf8_lossy(&escape_encode(value.as_bytes())[..]).to_string();
        let mut j = Self {
            headers: self.headers,
            payload: self.payload,
        };
        j.headers.insert(String::from(key), String::from(value));
        return j;
    }

    pub fn payload(self, payload: &[u8]) -> Self {
        // let payload = escape_encode(payload);
        let mut j = Self {
            headers: self.headers,
            payload: self.payload,
        };
        j.payload = payload.to_vec();
        return j;
    }

    pub fn to_network_stream(&self) -> Vec<u8> {
        let mut buf = vec![];
        for (k, v) in &self.headers {
            let mut a1 = Vec::new();
            a1.clone_from(&k.as_bytes().to_vec());
            a1 = escape_encode(&a1[..]);

            let mut a2 = Vec::new();
            a2.clone_from(&v.as_bytes().to_vec());
            a2 = escape_encode(&a2[..]);

            buf.append(&mut a1);

            if a2.len() != 0 {
                buf.append(&mut b":".to_vec());
                buf.append(&mut a2);
            }
            buf.append(&mut b"\n".to_vec());
        }
        buf.append(&mut b"\n".to_vec());

        let mut a1 = Vec::new();
        a1.clone_from(&self.payload.to_vec());
        a1 = escape_encode(&a1[..]);

        buf.append(&mut a1);
        buf.append(&mut b"\\".to_vec());
        buf
    }
}

impl PartialEq for HttpLikeData {
    fn eq(&self, other: &Self) -> bool {
        self.headers == other.headers && self.payload == other.payload
    }
    fn ne(&self, other: &Self) -> bool {
        self.headers != other.headers || self.payload != other.payload
    }
}