#![allow(unused)]

//! Drive the scraping process.

use std::iter;

use crate::err;
use crate::extract;
use crate::http;

use hyper::Uri;
use log::{info,warn,error};

/// Store fetched results, which come as key/value pairs, somewhere.
#[async_trait::async_trait]
pub trait Storage<T: Send> {
    async fn store(&mut self, d: Box<dyn Iterator<Item=T> + Send>) ->Result<(), err::HTTPError>;
}

/// An Extractor retrieves information from a Document.
pub trait Extractor<T: Send> {
    fn extract(&mut self, uri: &Uri, doc: &extract::Document) -> Vec<T> {
        vec![]
    }
    /// Return pages to fetch based on a fetched document.
    fn next_sites(&mut self, uri: &Uri, doc: &extract::Document) -> Vec<Uri>;
}

/// The Queue manages and prioritizes order and volume of sites to fetch.
#[async_trait::async_trait]
pub trait Queue {
    /// Add a site to the queue.
    async fn add(&mut self, uris: &[Uri]) -> Result<(), err::HTTPError>;
    /// Returns a site to scrape next.
    async fn next(&mut self) -> Result<Option<Uri>, err::HTTPError>;
}

/// DriverLogic holds the driven implementation. The members tell the driver what to fetch, and
/// what and how to store it.
pub struct DriverLogic<T> {
    pub store: Box<dyn Storage<T>>,
    pub extract: Box<dyn Extractor<T>>,
    pub queue: Box<dyn Queue>,
}

pub struct Driver<T> {
    https: http::HTTPS,
    logic: DriverLogic<T>,
}

impl<T: 'static + Send> Driver<T> {
    /// Create a new Driver instance.
    pub fn new(logic: DriverLogic<T>, https: Option<http::HTTPS>) -> Driver<T> {
        Driver { https: https.unwrap_or(http::HTTPS::new()), logic: logic }
    }

    /// Run Driver a single step, i.e. first explore, then process one page. Returns true if a page
    /// was processed.
    pub async fn drive(&mut self) -> Result<bool, err::HTTPError> {
        let next = self.logic.queue.next().await?;
        info!("Next URL: {:?}", next);

        if let Some(uri) = next {
            info!("Starting fetch of {}", uri);
            let resp = self.https.get(&uri).await?;
            let doc = extract::parse_response(resp)?;
            let extracted = self.logic.extract.extract(&uri, &doc);
            self.logic.store.store(Box::new(extracted.into_iter()));
            let next_urls = self.logic.extract.next_sites(&uri, &doc);
            info!("Appended URIs after fetch: {:?}", next_urls);
            self.logic.queue.add(&next_urls);
            return Ok(true);
        } else {
            Ok(false)
        }
    }
}

