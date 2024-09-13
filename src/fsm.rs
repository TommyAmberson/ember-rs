use std::time::Duration;

use anyhow::{bail, Context, Result};
use btleplug::api::bleuuid::BleUuid;
use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use uom::si::temperature_interval::degree_celsius;

use crate::mug::Mug;

pub(crate) enum App<'a> {
    Scanning {
        central: &'a Adapter,
    },
    Connecting {
        central: &'a Adapter,
        device: Peripheral,
    },
    Connected {
        central: &'a Adapter,
        mug: Mug,
    },
}

impl App<'_> {
    pub async fn tick(self) -> Result<Self> {
        match self {
            App::Scanning { central } => {
                // Each adapter has an event stream, we fetch via events(),
                // simplifying the type, this will return what is essentially a
                // Future<Result<Stream<Item=CentralEvent>>>.
                let mut events = central.events().await?;

                println!("scan");
                // start scanning for devices
                central.start_scan(ScanFilter::default()).await?;

                // Print based on whatever the event receiver outputs. Note that the event
                // receiver blocks, so in a real program, this should be run in its own
                // thread (not task, as this library does not yet use async channels).
                while let Some(event) = events.next().await {
                    match event {
                        CentralEvent::DeviceDiscovered(id) => {
                            let device = central.peripheral(&id).await?;
                            println!("found device: {:?}", device.address());
                            let properties = device.properties().await?;
                            // println!("props: {properties:?}");
                            let local_name = &properties
                                .context("Failed to get properties from device")?
                                .local_name;
                            println!("name: {local_name:?}");
                            if local_name.iter().any(|name| name.contains("Ember")) {
                                central.stop_scan().await?;
                                return Ok(Self::Connecting { central, device });
                            }
                        }
                        _other => {
                            // println!("{:?}", other);
                        }
                    }
                }
                central.stop_scan().await?;
                bail!("Failed to scan for devices");
            }
            App::Connecting { central, device } => {
                println!("connecting");
                if !device.is_connected().await.unwrap_or(false) {
                    device.connect().await?;
                }
                println!("discovering services");
                device.discover_services().await?;
                let mug = Mug::try_from(device)?;
                Ok(Self::Connected { central, mug })
            }
            App::Connected { central, mug } => {
                if !mug.connected().await {
                    return Ok(Self::Scanning { central });
                }

                println!("get temp");
                let temp = mug.get_current_temp().await?;
                println!("temp: {:?}", temp.get::<degree_celsius>());

                tokio::time::sleep(Duration::from_secs(5)).await;

                Ok(Self::Connected { central, mug })
            }
        }
    }
}
