use tokio::net::TcpStream;

use futures::future::Either;
use gumdrop::Options;
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
    println!("{:#?}", opts);

    use std::net::ToSocketAddrs;

    use futures::stream::futures_unordered::FuturesUnordered;
    let mut futs = FuturesUnordered::new();

    let tcp_futs = opts
        .tcp
        .iter()
        .flat_map(|s| s.to_socket_addrs().expect("valid socket addr"))
        .map(|socket_addr| Either::Left(wait_for_socket(socket_addr)));

    let http_futs = opts
        .http
        .iter()
        .map(|x| Url::parse(x).expect("valid HTTP URL"))
        .map(|url| Either::Right(wait_for_http(url)));

    futs.extend(tcp_futs);
    futs.extend(http_futs);

    use futures::stream::StreamExt;
    while let Some(result) = futs.next().await {
        result
    }

    Ok(())
}

async fn wait_for_socket(socket: std::net::SocketAddr) {
    loop {
        match TcpStream::connect(socket).await {
            Ok(_) => return,
            Err(_) => continue,
        };
    }
}

async fn wait_for_http(url: Url) {
    loop {
        match reqwest::get(url.as_str()).await {
            Ok(response) => {
                if response.status().is_success() {
                    return;
                } else {
                    continue;
                }
            }
            Err(_) => continue,
        };
    }
}
