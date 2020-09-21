//! Implementations of common traits that are useful to plug together a Driver.

use crate::driver;
use crate::err::HTTPError;

use log::{info};

pub struct DebuggingStorage { }

#[derive(Debug)]
pub struct ScrapedPrice {
    pub item: String,
    pub price: String,
    pub note: i32,
}

#[async_trait::async_trait]
impl driver::Storage<ScrapedPrice> for DebuggingStorage {
    async fn store(&mut self, all: Box<dyn Iterator<Item=ScrapedPrice> + Send>) -> Result<(), HTTPError> {
        info!("STORAGE: Received {:?}", all.collect::<Vec<ScrapedPrice>>());
        Ok(())
    }
}

