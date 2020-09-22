#![allow(unused)]

use crate::err::{logic_err, HTTPError};
use crate::http;

use std::iter;

use log::info;
use scraper::Html;

/// A fetched document is given to the Extractor which gets information from it and returns the
/// storable data. The underlying logic is implemented by the `scraper` crate.
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
    /// For every CSS selector in `selectors`, return a vec of contents in that selector.
    pub fn get_contents(&self, selectors: &[&str]) -> Result<Vec<Vec<String>>, HTTPError> {
        let mut r = Vec::with_capacity(selectors.len());
        for sel in selectors {
            let selector = parse_selector(sel)?;
            let selected = self.html.select(&selector);

            let mut values = vec![];
            for e in selected {
                values.push(e.inner_html());
            }
            r.push(values);
        }
        Ok(r)
    }
    /// For a selector, return a vec of contents for the selector.
    pub fn get_content(&self, selector: &str) -> Result<Vec<String>, HTTPError> {
        let v = self.get_contents(&[selector])?;
        Ok(v[0].clone())
    }
    /// For the elements described by selector, return the attributes
    pub fn get_attr(&self, selector: &str, attr: &str) -> Result<Vec<String>, HTTPError> {
        let selector = parse_selector(selector)?;
        let sel = self.html.select(&selector);
        let mut fetched = vec![];
        for item in sel {
            fetched.push(item.value().attr(attr).unwrap_or("").to_string());
        }
        Ok(fetched)
    }
}

fn parse_selector(sel: &str) -> Result<scraper::Selector, HTTPError> {
    scraper::Selector::parse(sel)
        .map_err(|_| HTTPError::LogicError(format!("failed to parse selector {}", sel)))
}

#[cfg(test)]
mod tests {
    use super::Document;

    use std::iter;

    #[test]
    fn test_document() {
        let content = String::from_utf8(std::fs::read("audiophil_sony.html").unwrap()).unwrap();
        let ex = Document::new(&content);
        let mut data = ex.get_contents(&[".bez.neu", ".preis strong"]).unwrap();
        let prices = data.pop().unwrap();
        let descs = data.pop().unwrap();
        let zipped: Vec<(String, String)> = descs
            .into_iter()
            .zip(prices)
            .map(|(desc, price)| (desc.trim().to_string(), price.trim().to_string()))
            .collect();
        println!("{:?}", zipped);

        let links = ex.get_attr("a", "href").unwrap();
        println!("All links: {:?}", links);
    }
}
