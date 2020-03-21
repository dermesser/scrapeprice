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
    pub fn get_field(&self, selector: &str) {
        let selector = scraper::Selector::parse(selector).unwrap();
        let selected = self.html.select(&selector);
        for e in selected {
            println!("selected: {}", e.inner_html());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Extract;

    #[test]
    fn test_extract() {
        let content = String::from_utf8(std::fs::read("audiophil_sony.html").unwrap()).unwrap();
        let ex = Extract::new(&content);
        ex.get_field(".bez.neu");
        ex.get_field(".preis strong");
    }
}
