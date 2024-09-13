use std::time::Duration;

use anyhow::Result;
use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::platform::Manager;
use tokio::time;
use uom::si::temperature_interval::degree_celsius;

use crate::mug::Mug;

mod mug;

#[tokio::main]
async fn main() -> Result<()> {
    println!("new manager");
    let manager = Manager::new().await?;

    println!("get adapters");
    // get the first bluetooth adapter
    let adapters = manager.adapters().await?;
    let central = adapters.first().unwrap();

    println!("scan");
    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;
    // instead of waiting, you can use central.events() to get a stream which will
    // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    time::sleep(Duration::from_secs(5)).await;

    println!("get mug");
    let mug = Mug::find_mug(central).await?;

    println!("get temp");
    let temp = mug.get_current_temp().await?;

    println!("temp: {:?}", temp.get::<degree_celsius>());

    Ok(())
}
