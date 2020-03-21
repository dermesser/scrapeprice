mod err;
mod extract;
mod http;

use log::{info, warn};
use env_logger;

async fn test_fetch_page() -> hyper::Result<()> {
    let mut cl = http::HTTPS::new();
    let res = cl.get("https://audiophil-foto.de/de/shop/kameras/sony/".parse::<hyper::Uri>().unwrap()).await.unwrap();
    info!("Fetch 1 was {}", res.status);
    let res = cl.get("https://audiophil-foto.de/de/shop/kameras/nikon/".parse::<hyper::Uri>().unwrap()).await.unwrap();
    info!("Fetch 2 was {}", res.status);

    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env().filter(None, log::LevelFilter::Info).init();

    info!("scrapeprice: init");
    test_fetch_page().await.unwrap();
}
