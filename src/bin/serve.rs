use clap::{value_parser, Arg, Command};
use simple_logger::SimpleLogger;
use simple_webserver::http_server::*;
use simple_webserver::*;

fn main() {
    let matches = Command::new("Simple Rust HTTP Server")
        .version("1.0")
        .about("Simple webserver that implements the HTTP/1.0 protocal and serves files from your local directory")
        .arg(Arg::new("port").value_parser(value_parser!(u16)).default_value("8080").short('p').long("port"))
        .arg(Arg::new("protocol").default_value("HTTP/1.0").long("protocol"))
        .arg(Arg::new("bind").default_value("127.0.0.1").short('b').long("bind"))
        .arg(Arg::new("directory").default_value("./").short('d').long("directory"))
        .arg(Arg::new("poolsize").value_parser(value_parser!(usize)).default_value("5").short('s').long("poolsize"))
        .get_matches();

    let port = *matches.get_one::<u16>("port").unwrap();
    let protocol = matches.get_one::<String>("protocol").unwrap().to_string();
    let bind = matches.get_one::<String>("bind").unwrap().to_string();
    let directory = matches.get_one::<String>("directory").unwrap().to_string();
    let poolsize = *matches.get_one::<usize>("poolsize").unwrap();
    let args = Opts {
        port,
        bind,
        protocol,
        directory,
    };

    // Initialize a new logger
    SimpleLogger::new().init().unwrap();
    log::info!("Logging started...");

    //let http_server = HTTPServer::new(HTTPServerClass::Simple, args, None);
    //let http_server = HTTPServer::new(HTTPServerClass::Threaded, args, None);
    let http_server = HTTPServer::new(HTTPServerClass::ThreadPooled(poolsize), args, None);

    http_server.serve_forever();
}
