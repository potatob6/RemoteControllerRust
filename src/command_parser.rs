#[derive(Debug)]
pub struct HttpLikeData {
    pub headers: Vec<(String, String)>,
    pub payload: Vec<u8>,
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
                            // If there is not on the table, you can decide add the buf into result buf
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

// Test Complete[√]
pub fn multi_command_parser<'a>(cmd: &'a [u8]) -> HttpLikeData {
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

    for (i, k) in iters.enumerate() {
        if i == 0 {
            ref_headers = k;
        } else {
            ref_payload = k;
        }
    }

    let mut vec_headers = vec![];
    ref_headers = &ref_headers[..ref_headers.len() - 1];
    for k in ref_headers.split(|num| *num == b'\n') {
        if k.len() != 0 {
            let mut iter1 = k.split(|num| *num == b':');
            let split_kv = (
                String::from_utf8_lossy(
                    &escape_decode(iter1.next().unwrap())[..]
                ).to_string(), 
                String::from_utf8_lossy(
                    &escape_decode(iter1.next().unwrap())[..]
                ).to_string());
            vec_headers.push(split_kv);
        }
    }

    HttpLikeData { 
        headers: vec_headers, 
        payload: escape_decode(ref_payload),
    }
    // Parse command and data

}