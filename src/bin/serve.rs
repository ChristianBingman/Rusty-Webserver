use simple_logger::SimpleLogger;
use simple_webserver::http_server::*;
use simple_webserver::*;

fn main() {
    let args = Opts::default();

    // Initialize a new logger
    SimpleLogger::new().init().unwrap();
    log::info!("Logging started...");

    //let http_server = HTTPServer::new(HTTPServerClass::Simple, args, None);
    //let http_server = HTTPServer::new(HTTPServerClass::Threaded, args, None);
    let http_server = HTTPServer::new(HTTPServerClass::ThreadPooled, args, None);

    http_server.serve_forever();
}
