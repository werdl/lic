mod server;
mod client;

use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
    Server {
        #[clap(short = 'o', long, default_value = "localhost")]
        host: String,
        #[clap(short, long, default_value = "1742")]
        port: u16,
        #[clap(short, long, default_value = "4")]
        threads: usize,
        #[clap(short, long, default_value = "100")]
        max_connections: usize,
    },
    Client {
        #[clap(short = 'o', long, default_value = "localhost")]
        host: String,
        #[clap(short, long, default_value = "1742")]
        port: u16,
    }
}

fn main() {
    let opts: Opts = Opts::parse();
    match opts.subcmd {
        SubCommand::Server { host, port, threads, max_connections } => {
            let mut server = server::Server::new("localhost:1742");

            server.run();
        }
        SubCommand::Client { host, port } => {
            let mut client = client::Client::new(client::ClientOpts {
                host,
                port,
            }).unwrap();
            client.run().unwrap();
        }
    }
}
