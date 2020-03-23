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
pub trait Storage<T> {
    async fn store(&mut self, d: Vec<T>) ->Result<(), err::HTTPError>;
}

/// Return Uris to explore, both as initial set and for every fetched page.
pub trait Explorer {
    /// Return pages to fetch in any case, e.g. time-based. Called on every iteration of the
    /// driver.
    fn idle(&mut self) -> Vec<Uri>;
    /// Return pages to fetch based on a fetched document.
    fn next(&mut self, uri: &Uri, doc: &extract::Document) -> Vec<Uri>;
}

/// An Extractor retrieves information from a Document.
pub trait Extractor<T> {
    fn extract(&mut self, uri: &Uri, doc: &extract::Document) -> Vec<T> {
        vec![]
    }
}

/// DriverLogic holds the driven implementation. The members tell the driver what to fetch, and
/// what and how to store it.
pub struct DriverLogic<T> {
    pub explore: Box<dyn Explorer>,
    pub store: Box<dyn Storage<T>>,
    pub extract: Box<dyn Extractor<T>>,
}

pub struct Driver<T> {
    https: http::HTTPS,
    logic: DriverLogic<T>,

    // This could be made into a more elaborate scheduler.
    queue: Vec<Uri>,
}

impl<T> Driver<T> {
    /// Create a new Driver instance.
    pub fn new(logic: DriverLogic<T>, https: Option<http::HTTPS>) -> Driver<T> {
        Driver { https: https.unwrap_or(http::HTTPS::new()), logic: logic, queue: Vec::with_capacity(64) }
    }

    /// Run Driver a single step, i.e. first explore, then process one page. Returns true if a page
    /// was processed.
    pub async fn drive(&mut self) -> Result<bool, err::HTTPError> {
        let new = self.logic.explore.idle();
        info!("Appended URIs to queue: {:?}", new);
        self.queue.extend(new.into_iter());

        if let Some(uri) = self.queue.pop() {
            info!("Starting fetch of {}", uri);
            let resp = self.https.get(&uri).await?;
            let doc = extract::parse_response(resp)?;
            let extracted = self.logic.extract.extract(&uri, &doc);
            self.logic.store.store(extracted);
            let next = self.logic.explore.next(&uri, &doc);
            info!("Appended URIs after fetch: {:?}", next);
            self.queue.extend(next);
            return Ok(true);
        } else {
            Ok(false)
        }
    }
}

