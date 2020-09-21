mod audiophil;

use scrapeprice::{driver, util};

use log::{info, warn};
use env_logger;
use tokio;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env().filter(None, log::LevelFilter::Info).init();

    info!("scrapeprice: init");
    //test_fetch_page().await.unwrap();

    let logic = driver::DriverLogic {
        explore: Box::new(audiophil::AudiophilExplorer::new()),
        store: Box::new(util::DebuggingStorage {}),
        extract: Box::new(audiophil::AudiophilItemPriceExtractor {}),
    };
    let mut driver = driver::Driver::new(logic, None);

    let mut ival = tokio::time::interval(tokio::time::Duration::from_millis(2000));

    loop {
        ival.tick().await;
        if let Err(e) = driver.drive().await {
            warn!("Error from driver:  {}", e);
        }
    }
}
