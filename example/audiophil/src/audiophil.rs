use scrapeprice::err::HTTPError;
use scrapeprice::util::ScrapedPrice;

use std::collections::HashSet;
use std::collections::LinkedList;
use std::iter::FromIterator;

use scrapeprice::{driver, extract};

use hyper::Uri;
use log::info;
use rex_regex as rex;

pub struct AudiophilItemPriceExtractor {}

fn substring(s: String, (start, len): (usize, usize)) -> String {
    String::from_iter(s.chars().skip(start).take(len))
}

impl driver::Extractor<ScrapedPrice> for AudiophilItemPriceExtractor {
    fn next_sites(&mut self, _uri: &Uri, _doc: &extract::Document) -> Vec<Uri> {
        vec![]
    }
    fn extract(&mut self, uri: &Uri, doc: &extract::Document) -> Vec<ScrapedPrice> {
        info!("Extracting info from {}", uri);
        let mut data = doc.get_contents(&[".bez.neu", ".preis strong"]).unwrap();
        let prices = data.pop().unwrap();
        let descs = data.pop().unwrap();

        let onlytext = rex::compile("^([a-zA-Z0-9€\\.,+/ -]+)").unwrap();

        let zipped = descs
            .into_iter()
            .zip(prices)
            .map(|(desc, price)| (desc.trim().to_string(), price.trim().to_string()))
            .map(move |(desc, price)| {
                let desc2;
                let price2;
                let (ok, descmatch) = rex::match_re(&onlytext, &desc);
                if ok {
                    desc2 = substring(desc, descmatch[0]);
                } else {
                    desc2 = desc;
                }
                let (ok, pricematch) = rex::match_re(&onlytext, &price);
                if ok {
                    price2 = substring(price, pricematch[0]);
                } else {
                    price2 = price;
                }

                ScrapedPrice {
                    item: desc2,
                    price: price2,
                    note: 44,
                }
            })
            .collect();
        info!("Extracted {:?}", zipped);
        zipped
    }
}

pub struct AudiophilQueue {
    q: LinkedList<Uri>,
    visited: HashSet<Uri>,
}

impl AudiophilQueue {
    pub fn new() -> AudiophilQueue {
        let initial: Vec<Uri> = vec![
            "https://audiophil-foto.de/de/shop/kameras/sony/",
            "https://audiophil-foto.de/de/shop/kameras/pentax-ricoh/",
            "https://audiophil-foto.de/de/shop/kameras/leica/",
            "https://audiophil-foto.de/de/shop/objektive/sony/",
            "https://audiophil-foto.de/de/shop/objektive/zeiss/",
            "https://audiophil-foto.de/de/shop/objektive/sigma/",
        ]
        .into_iter()
        .map(|s| s.parse::<Uri>().unwrap())
        .collect();
        AudiophilQueue {
            q: LinkedList::from_iter(initial.into_iter()),
            visited: HashSet::new(),
        }
    }
}

#[async_trait::async_trait]
impl driver::Queue for AudiophilQueue {
    async fn add(&mut self, uris: &[Uri]) -> Result<(), HTTPError> {
        for u in uris {
            if !self.visited.contains(u) {
                self.q.push_back(u.clone());
            }
        }
        Ok(())
    }
    async fn next(&mut self) -> Result<Option<Uri>, HTTPError> {
        if !self.q.is_empty() {
            return Ok(self.q.pop_front());
        }
        Ok(None)
    }
    async fn visited(&mut self, uri: &Uri) -> Result<(), HTTPError> {
        self.visited.insert(uri.clone());
        Ok(())
    }
}
