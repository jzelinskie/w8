use tokio::net::TcpStream;

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

    use futures::stream::futures_unordered::FuturesUnordered;
    let mut futs = FuturesUnordered::new();

    use std::net::ToSocketAddrs;
    futs.extend(
        opts.tcp
            .iter()
            .flat_map(|s| s.to_socket_addrs().expect("valid socket addr")) // This does IPv4 AND IPv6, when it should not.
            .map(|s| {
                println!("{:#?}", s);
                s
            })
            .map(|socket_addr| tokio::spawn(wait_for_socket(socket_addr))),
    );

    futs.extend(
        opts.http
            .iter()
            .map(|x| Url::parse(x).expect("valid HTTP URL"))
            .map(|s| {
                println!("{:#?}", s);
                s
            })
            .map(|url| tokio::spawn(wait_for_http(url))),
    );

    use futures::stream::StreamExt;
    while let Some(_) = futs.next().await {
        println!("did it");
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
                    println!("success");
                    return;
                } else {
                    continue;
                }
            }
            Err(_) => continue,
        };
    }
}
