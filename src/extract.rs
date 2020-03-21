use crate::err::{logic_err, HTTPError};
use crate::http;

use log::info;
use scraper::Html;

pub struct Extract {
    html: Html,
}

pub fn parse_response(r: http::GetResponse) -> Extract {
    let content = http::bytes_to_str(r.body).unwrap();
    let doc = Html::parse_document(content.as_str());
    Extract { html: doc }
}

impl Extract {
    fn new(content: &str) -> Extract {
        Extract {
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
                println!("selected: {}", e.inner_html());
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

#[cfg(test)]
mod tests {
    use super::Extract;

    use std::iter;

    #[test]
    fn test_extract() {
        let content = String::from_utf8(std::fs::read("audiophil_sony.html").unwrap()).unwrap();
        let ex = Extract::new(&content);
        let mut data = ex.get_fields(&[".bez.neu", ".preis strong"]).unwrap();
        let prices = data.pop().unwrap();
        let descs = data.pop().unwrap();
        let zipped: Vec<(String, String)> = descs.into_iter().zip(prices).map(|(desc, price)| {
            (desc.trim().to_string(), price.trim().to_string())
        }).collect();
        println!("{:?}", zipped);
    }
}
