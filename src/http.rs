use std::collections::HashMap;
use std::convert::{Into, TryFrom};
use std::error::Error;

use http;
use hyper;
use robots_txt::{matcher, parts::robots};

type HyperHTTPS =
    hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>;

fn new_hyper_client() -> HyperHTTPS {
    let b = hyper::Client::builder();
    b.build(hyper_rustls::HttpsConnector::new())
}

pub fn bytes_to_str(b: hyper::body::Bytes) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(b.as_ref().to_vec())
}

fn robots_ok(robots_txt: &str, uri: &hyper::Uri) -> bool {
    let r = robots::Robots::from_str(robots_txt);
    let m = matcher::SimpleMatcher::new(&r.choose_section("*").rules);
    let m2 = matcher::SimpleMatcher::new(&r.choose_section("scrapeprice").rules);
    m.check_path(uri.path()) && m2.check_path(uri.path())
}

pub struct HTTPS {
    client: HyperHTTPS,
    agent: String,
    robots_txt_cache: HashMap<String, String>,
}

#[derive(Debug)]
pub struct GetResponse {
    pub status: hyper::StatusCode,
    pub body: hyper::body::Bytes,
}

impl HTTPS {
    pub fn new() -> HTTPS {
        HTTPS {
            client: new_hyper_client(),
            agent: "scrapeprice (lbo@spheniscida.de)".to_string(),
            robots_txt_cache: HashMap::new(),
        }
    }

    pub async fn get(&mut self, uri: hyper::Uri) -> Result<GetResponse, Box<dyn Error>> {
        if let Ok(true) = self.robots_ok(&uri).await {
            return self
                .get_nocheck(uri)
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error>);
        }
        unimplemented!()
    }

    pub async fn get_nocheck(&self, uri: hyper::Uri) -> hyper::Result<GetResponse> {
        use follow_redirects::ClientExt;

        let rq = self.make_request(&uri);
        let cl = self.client.follow_redirects();
        let body = cl.request(rq).await?;
        let status = body.status();
        let bytes = hyper::body::to_bytes(body).await?;
        println!("GET {:?} => {}", uri, status);
        Ok(GetResponse {
            status: status,
            body: bytes,
        })
    }

    async fn robots_ok(&mut self, uri: &hyper::Uri) -> hyper::Result<bool> {
        let host = uri.host().unwrap_or("_");
        let parts = host.to_string().split(".").collect::<Vec<&str>>();
        println!("checking robots.txt for {}", host);
        match self.robots_txt_cache.get(host) {
            Some(e) => Ok(robots_ok(e, uri)),
            _ => {
                let mut robots_uri = hyper::Uri::builder()
                    .authority(host)
                    .scheme(uri.scheme_str().unwrap_or("http"))
                    .path_and_query("/robots.txt")
                    .build()
                    .unwrap();
                let resp = self.get_nocheck(robots_uri).await?;
                println!("{:?}", resp.body);
                let robots = bytes_to_str(resp.body).unwrap();
                println!("{}", robots);
                let is_ok = robots_ok(&robots, uri);
                self.robots_txt_cache.insert(host.to_string(), robots);

                Ok(is_ok)
            }
        }
    }

    fn make_request<T>(&self, uri: T) -> hyper::Request<hyper::Body>
    where
        hyper::Uri: TryFrom<T>,
        <hyper::Uri as TryFrom<T>>::Error: Into<http::Error>,
    {
        let body = hyper::body::Body::empty();
        hyper::Request::builder()
            .uri(uri)
            .header("User-Agent", &self.agent)
            .method(hyper::Method::GET)
            .body(body)
            .unwrap()
    }
}
