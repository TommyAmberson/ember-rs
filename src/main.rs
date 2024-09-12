use std::time::Duration;

use anyhow::{Context, Result};
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use tokio::time;

#[tokio::main]
async fn main() -> Result<()> {
    let manager = Manager::new().await?;

    // get the first bluetooth adapter
    let adapters = manager.adapters().await?;
    let central = adapters.first().unwrap();

    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;
    // instead of waiting, you can use central.events() to get a stream which will
    // notify you of new devices, for an example of that see examples/event_driven_discovery.rs
    time::sleep(Duration::from_secs(2)).await;

    // find the device we're interested in
    let mug = find_mug(central)
        .await
        .with_context(|| "Failed to find mug")?;

    // connect to the device
    mug.connect().await?;

    // discover services and characteristics
    mug.discover_services().await?;

    // find the characteristic we want
    let chars = mug.characteristics();
    let cmd_char = chars
        .iter()
        .find(|c| c.uuid.to_string() == "fc540002-236c-4c94-8fa9-944a3e5353fa")
        .unwrap();

    println!("{cmd_char}");

    let temp = mug.read(cmd_char).await?;
    let temp: [u8; 2] = temp
        .try_into()
        .map_err(|temp| anyhow::anyhow!("Couldn't map {temp:?} to u64"))?;
    let temp = u16::from_le_bytes(temp);
    let temp = temp as f32 / 100.0;

    println!("{temp}");

    Ok(())
}

async fn find_mug(central: &Adapter) -> Option<Peripheral> {
    for p in central.peripherals().await.unwrap() {
        if p.properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .iter()
            .any(|name| name.contains("Ember"))
        {
            return Some(p);
        }
    }
    None
}
