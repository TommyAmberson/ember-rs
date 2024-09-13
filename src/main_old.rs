use anyhow::Result;
use btleplug::api::bleuuid::BleUuid;
use btleplug::api::{Central, CentralEvent, Manager as _, ScanFilter};
use btleplug::platform::Manager;
use uom::si::temperature_interval::degree_celsius;

use crate::fsm::App;
use crate::mug::Mug;

mod fsm;
mod mug;

#[tokio::main]
async fn main() -> Result<()> {
    println!("new manager");
    let manager = Manager::new().await?;

    println!("get adapters");
    // get the first bluetooth adapter
    let adapters = manager.adapters().await?;
    let central = adapters.first().unwrap();

    let mut fsm = App::Scanning { central };
    loop {
        fsm = fsm.tick().await?;
    }

    //     // Each adapter has an event stream, we fetch via events(),
    //     // simplifying the type, this will return what is essentially a
    //     // Future<Result<Stream<Item=CentralEvent>>>.
    //     let mut events = central.events().await?;
    //
    //     println!("scan");
    //     // start scanning for devices
    //     central.start_scan(ScanFilter::default()).await?;
    //
    //     // Print based on whatever the event receiver outputs. Note that the event
    //     // receiver blocks, so in a real program, this should be run in its own
    //     // thread (not task, as this library does not yet use async channels).
    //     while let Some(event) = events.next().await {
    //         match event {
    //             CentralEvent::DeviceDiscovered(id) => {
    //                 println!("DeviceDiscovered: {:?}", id);
    //             }
    //             CentralEvent::DeviceConnected(id) => {
    //                 println!("DeviceConnected: {:?}", id);
    //             }
    //             CentralEvent::DeviceDisconnected(id) => {
    //                 println!("DeviceDisconnected: {:?}", id);
    //             }
    //             CentralEvent::ManufacturerDataAdvertisement {
    //                 id,
    //                 manufacturer_data,
    //             } => {
    //                 println!(
    //                     "ManufacturerDataAdvertisement: {:?}, {:?}",
    //                     id, manufacturer_data
    //                 );
    //             }
    //             CentralEvent::ServiceDataAdvertisement { id, service_data } => {
    //                 println!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
    //             }
    //             CentralEvent::ServicesAdvertisement { id, services } => {
    //                 let services: Vec<String> =
    //                     services.into_iter().map(|s| s.to_short_string()).collect();
    //                 println!("ServicesAdvertisement: {:?}, {:?}", id, services);
    //             }
    //             _ => {}
    //         }
    //     }
    //
    //     println!("get mug");
    //     let mug = Mug::find_mug(central).await?;
    //
    //     println!("get temp");
    //     let temp = mug.get_current_temp().await?;
    //
    //     println!("temp: {:?}", temp.get::<degree_celsius>());
    //
    //
}
