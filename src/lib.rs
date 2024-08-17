pub mod http10;
mod file;
pub mod threadpool;
use core::str;
use std::{io::{Read, Write}, net::TcpStream};

use http10::{handle_request, request::HTTPRequest};

pub fn handle_client(mut stream: TcpStream) {
    let local: String = match stream.local_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "Invalid Address".to_string(),
    };
    let remote: String = match stream.peer_addr() {
        Ok(addr) => addr.to_string(),
        Err(_) => "Invalid Address".to_string(),
    };
    let mut request: Vec<u8> = Vec::new();
    let mut buf = [0u8; 4096];
    log::info!("Received connection from {} to {}", remote, local);
    while HTTPRequest::try_from(&request).is_err() {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if let Ok(parsed_data) = str::from_utf8(&buf[..n]) {
                    log::info!("Received: {}", parsed_data);
                    request.append(buf[..n].to_vec().as_mut());
                } else {
                    request.append(buf[..n].to_vec().as_mut());
                }
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => (),
            Err(_) => break
        }
    }
    stream.write_all(handle_request(request).as_bytes().as_slice()).unwrap();
}
