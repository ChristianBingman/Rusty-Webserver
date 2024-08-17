use clap::Parser;
use simple_logger::SimpleLogger;
use simple_webserver::*;
use std::net::TcpListener;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to bind to
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Address to bind to
    #[arg(short, long, default_value = "127.0.0.1")]
    bind: String,

    /// Directory to serve
    #[arg(short, long, default_value = "./")]
    directory: String,

    /// Protocol to use (supports HTTP 1.0)
    #[arg(long, default_value = "HTTP/1.0")]
    protocol: String,
}

fn main() {
    let args = Args::parse();

    // Initialize a new logger
    SimpleLogger::new().init().unwrap();
    log::info!("Logging started...");

    let mut tpq = threadpool::ThreadPoolQ::new(2, handle_client);
    if let Ok(listener) = TcpListener::bind(format!("{}:{}", args.bind, args.port)) {
        log::warn!("Started listening on {}:{}...", args.bind, args.port);
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => tpq.push_job(stream),
                Err(e) => log::error!("Connection failed with {e:?}"),
            }
        }
    } else {
        log::error!("Unable to start listening on 127.0.0.1:8080");
        panic!("Unable to bind port 8080 on 127.0.0.1")
    }
}
