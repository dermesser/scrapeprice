mod http;

use std::io;
use hyper;
use hyper_rustls;

use tokio::prelude::*;

async fn test_fetch_page() -> hyper::Result<()> {
    println!("testing!");
    let mut cl = http::HTTPS::new();
    let res = cl.get("https://audiophil-foto.de/de/shop/kameras/sony/".parse::<hyper::Uri>().unwrap()).await.unwrap();
    println!("{}\n{}", res.status, http::bytes_to_str(res.body).unwrap());

    Ok(())
}

#[tokio::main]
async fn main() {
    test_fetch_page().await.unwrap();
}
