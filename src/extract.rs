#![allow(unused)]

use crate::err::{logic_err, HTTPError};
use crate::http;

use std::iter;

use log::info;
use scraper::Html;

/// A fetched document is given to the Extractor which gets information from it and returns the
/// storable data.
pub struct Document {
    html: Html,
}

pub fn parse_response(r: http::GetResponse) -> Result<Document, HTTPError> {
    let content = http::bytes_to_str(r.body)?;
    let doc = Html::parse_document(content.as_str());
    Ok(Document { html: doc })
}

impl Document {
    fn new(content: &str) -> Document {
        Document {
            html: Html::parse_document(content),
        }
    }
    pub fn get_fields(&self, selectors: &[&str]) -> Result<Vec<Vec<String>>, HTTPError> {
        let mut r = Vec::with_capacity(selectors.len());
        for sel in selectors {
            let selector = scraper::Selector::parse(sel)
                .map_err(|_| HTTPError::LogicError(format!("failed to parse selector {}", sel)))?;
            let selected = self.html.select(&selector);

            let mut values = vec![];
            for e in selected {
                values.push(e.inner_html());
            }
            r.push(values);
        }
        Ok(r)
    }
    pub fn get_field(&self, selector: &str) -> Result<Vec<String>, HTTPError> {
        let v = self.get_fields(&[selector])?;
        Ok(v[0].clone())
    }
}

pub trait Extracted {
    fn all(&mut self) -> Box<dyn iter::Iterator<Item = (String, String)> + Send> {
        Box::new(iter::empty())
    }
}

pub trait Extractor {
    fn extract(&mut self, doc: &Document) -> Option<Box<dyn Extracted>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Document;

    use std::iter;

    #[test]
    fn test_document() {
        let content = String::from_utf8(std::fs::read("audiophil_sony.html").unwrap()).unwrap();
        let ex = Document::new(&content);
        let mut data = ex.get_fields(&[".bez.neu", ".preis strong"]).unwrap();
        let prices = data.pop().unwrap();
        let descs = data.pop().unwrap();
        let zipped: Vec<(String, String)> = descs
            .into_iter()
            .zip(prices)
            .map(|(desc, price)| (desc.trim().to_string(), price.trim().to_string()))
            .collect();
        println!("{:?}", zipped);
    }
}
