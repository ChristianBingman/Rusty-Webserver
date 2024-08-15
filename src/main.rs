mod http10;
mod file;
mod threadpool;
use core::{panic, str};
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}};

use http10::{handle_request, request::HTTPRequest};
use simple_logger::SimpleLogger;
use threadpool::tpqueue::ThreadPoolQ;

fn handle_client(stream: TcpStream) {
    let mut stream = stream.try_clone().unwrap();
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

fn main() {
    // Initialize a new logger
    SimpleLogger::new().init().unwrap();
    log::info!("Logging started...");

    let mut tpq = ThreadPoolQ::new(2, handle_client);
    if let Ok(listener) = TcpListener::bind("0.0.0.0:8080") {
        log::warn!("Started listening on 127.0.0.1:8080...");
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => tpq.push_job(stream),
                Err(e) => log::error!("Connection failed with {e:?}")
            }
        }
    } else {
        log::error!("Unable to start listening on 127.0.0.1:8080");
        panic!("Unable to bind port 8080 on 127.0.0.1")
    }
}
