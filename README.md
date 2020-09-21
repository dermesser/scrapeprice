# `scrapeprice`

is a small one-binary Rust framework for fetching and analyzing web pages. It
honors robots.txt and is based on tokio. The logic for specific websites is
implemented in Rust -- alternatively, a configuration-based system can be
implemented.

To implement your own scraper, you need to implement three traits according to
your needs; the implementations are stored in a `driver::DriverLogic` object.

- `driver::Explorer` tells the driver which URLs to visit. It returns a list of
URLs to initially visit and extracts new URLs from fetched documents.
- `driver::Extractor<T>` extracts items of type `T` from a fetched web page.
- `driver::Storage<T>` stores the items returned by an `Extractor`, whether in a
log file, CSV or other text format, or into a database.

Once you have implemented those traits, check out the `main.rs` file as example.
Not much more is needed now:

```rust
    let logic = driver::DriverLogic {    
        explore: Box::new(implem::YourExplorer::new()),    
        store: Box::new(implem::YourStorage {}),    
        extract: Box::new(implem::YourItemPriceExtractor {}),    
    };    
    let mut driver = driver::Driver::new(logic, None);    
     
    let mut ival = tokio::time::interval(tokio::time::Duration::from_millis(2000));    
    
    loop {    
        ival.tick().await;     
        if let Err(e) = driver.drive().await {    
            warn!("Error from driver:  {}", e);    
        }    
    } 
```

For an example, you can check out the `audiophil` module in the `examples`
directory, which scrapes the website of my local photo store.
