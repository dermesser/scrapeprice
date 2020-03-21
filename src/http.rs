use crate::err::HTTPError;

use std::collections::HashMap;
use std::convert::{Into, TryFrom};

use http;
use hyper;
use log::{info, warn, error};
use robots_txt::{matcher, parts::robots};

type HyperHTTPS =
    hyper::Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>, hyper::Body>;

fn new_hyper_client() -> HyperHTTPS {
    let b = hyper::Client::builder();
    b.build(hyper_rustls::HttpsConnector::new())
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

pub fn bytes_to_str(b: hyper::body::Bytes) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(b.as_ref().to_vec())
}

impl HTTPS {
    pub fn new() -> HTTPS {
        HTTPS {
            client: new_hyper_client(),
            agent: "scrapeprice (lbo@spheniscida.de)".to_string(),
            robots_txt_cache: HashMap::new(),
        }
    }

    pub async fn get(&mut self, uri: hyper::Uri) -> Result<GetResponse, HTTPError> {
        if let Ok(true) = self.robots_ok(&uri).await {
            return self.get_nocheck(uri).await;
        }
        unimplemented!()
    }

    pub async fn get_nocheck(&self, uri: hyper::Uri) -> Result<GetResponse, HTTPError> {
        let max_redirect: i32 = 10;
        let mut uri = uri;
        let host = uri.host().unwrap().to_string();

        for i in 0..max_redirect {
            let rq = self.make_request(&uri);
            let resp = self
                .client
                .request(rq)
                .await
                .map_err(HTTPError::HyperError)?;

            info!("({}) GET {:?} => {}", i, uri, resp.status());
            match resp.status().as_u16() {
                200 => {
                    let status = resp.status();
                    let bytes = hyper::body::to_bytes(resp)
                        .await
                        .map_err(HTTPError::HyperError)?;
                    return Ok(GetResponse {
                        status: status,
                        body: bytes,
                    });
                }
                301 | 302 | 303 | 307 | 308 => {
                    let loc = resp.headers().get("location").or(resp.headers().get("Location"));
                    if let Some(location) = loc {
                        uri = hyper::Uri::builder()
                            .authority(host.as_str())
                            .scheme(uri.scheme_str().unwrap_or("https"))
                            .path_and_query(location.to_str().unwrap())
                            .build()
                            .map_err(HTTPError::HttpError).unwrap();
                        info!("({}) GET 302 Redirect to: {}", i, uri);
                        continue;
                    } else {
                        warn!("redirect without location: {:?}", resp.headers());
                        return Err(HTTPError::LogicError(format!("redirect without location: {:?}", resp.headers())))
                    }
                }
                404 => return Err(HTTPError::StatusError(resp.status())),
                _ => {}
            }
        }
        Err(HTTPError::LogicError(format!(
            "exhausted redirects on {}",
            uri
        )))
    }

    async fn robots_ok(&mut self, uri: &hyper::Uri) -> Result<bool, HTTPError> {
        let host = uri.host().unwrap_or("_");
        info!("checking robots.txt for {}", host);
        match self.robots_txt_cache.get(host) {
            Some(e) => {
                let is_ok = robots_ok(e, uri);
                info!("cached robots.txt for {} ok? {}", host, is_ok);
                Ok(is_ok)
            }
            _ => {
                let robots_uri = hyper::Uri::builder()
                    .authority(host)
                    .scheme(uri.scheme_str().unwrap_or("http"))
                    .path_and_query("/robots.txt")
                    .build()
                    .unwrap();
                let resp = self.get_nocheck(robots_uri).await?;
                let robots = bytes_to_str(resp.body).unwrap();
                let is_ok = robots_ok(&robots, uri);
                self.robots_txt_cache.insert(host.to_string(), robots);
                info!("robots.txt for {} ok? {}", host, is_ok);
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
