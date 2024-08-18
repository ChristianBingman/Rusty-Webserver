use clap::Parser;
use simple_logger::SimpleLogger;
use simple_webserver::http_server::*;
use simple_webserver::*;
use std::net::TcpListener;

fn main() {
    let args = Opts::default();

    // Initialize a new logger
    SimpleLogger::new().init().unwrap();
    log::info!("Logging started...");

    let http_server = HTTPServer::new(HTTPServerClass::Simple, args, None);
    //let http_server = HTTPServer::new(HTTPServer::Class::Threaded, args);
    //let http_server = HTTPServer::new(HTTPServer::Class::ThreadPool, args);

    http_server.serve_forever();
}
