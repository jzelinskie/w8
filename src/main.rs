use tokio::net::TcpStream;

use gumdrop::Options;
use tracing::debug;
use url::Url;

#[derive(Debug, Options)]
struct W8Options {
    #[options(help = "print usage info")]
    help: bool,

    #[options(help = "enable verbose logging")]
    verbose: bool,

    #[options(help = "tcp sockets to be bound")]
    tcp: Vec<String>,

    #[options(help = "http endpoint to return 2xx")]
    http: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = W8Options::parse_args_default_or_exit();

    use tracing_subscriber::{fmt, EnvFilter};
    if opts.verbose {
        fmt().with_env_filter("w8=debug").init();
    } else {
        fmt().with_env_filter(EnvFilter::from_default_env()).init();
    }
    debug!(options = ?opts, "parsed command options");

    use futures::stream::futures_unordered::FuturesUnordered;
    let mut futs = FuturesUnordered::new();

    use std::net::ToSocketAddrs;
    futs.extend(
        opts.tcp
            .iter()
            .flat_map(|s| s.to_socket_addrs().expect("valid socket addr")) // This does IPv4 AND IPv6, when it should not.
            .map(|s| {
                debug!(socket = ?s, "parsed socketaddr");
                s
            })
            .map(|socket_addr| tokio::spawn(wait_for_socket(socket_addr))),
    );

    futs.extend(
        opts.http
            .iter()
            .map(|x| Url::parse(x).expect("valid HTTP URL"))
            .map(|u| {
                debug!(url = ?u, "parsed url");
                u
            })
            .map(|url| tokio::spawn(wait_for_http(url))),
    );

    use futures::stream::StreamExt;
    while let Some(result) = futs.next().await {
        debug!(result = ?result, "realized future");
    }

    Ok(())
}

async fn wait_for_socket(socket: std::net::SocketAddr) {
    loop {
        match TcpStream::connect(socket).await {
            Ok(_) => {
                debug!(socket = ?socket, "successfully connected to tcp socket");
                return;
            }
            Err(err) => {
                debug!(socket = ?socket, error = ?err, "failed connecting to tcp socket, retrying...");
                continue;
            }
        };
    }
}

async fn wait_for_http(url: Url) {
    loop {
        match reqwest::get(url.as_str()).await {
            Ok(response) => {
                if response.status().is_success() {
                    debug!(url = ?url, response = ?response, "successfully received 2xx from http endpoint");
                    return;
                } else {
                    debug!(url = ?url, response = ?response, "failed receiving 2xx from http endpoint, retrying...");
                    continue;
                }
            }
            Err(err) => {
                debug!(url = ?url, error = ?err, "failed getting http response, retrying...");
                continue;
            }
        };
    }
}
