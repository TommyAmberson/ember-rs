use std::collections::HashSet;

use anyhow::{anyhow, Context, Result};
use btleplug::api::{Central, Peripheral as _};
use btleplug::platform::{Adapter, Peripheral};
use uom::si::f32::TemperatureInterval;
use uom::si::temperature_interval::degree_celsius;

pub(crate) struct Mug {
    device: Peripheral,
    // chars: BTreeSet<Characteristic>,
}

impl Mug {
    const TARGET_TEMP: &'static str = "fc540003-236c-4c94-8fa9-944a3e5353fa";
    // const LED_COLOUR: &'static str = "fc540014-236c-4c94-8fa9-944a3e5353fa";
    const CURRENT_TEMP: &'static str = "fc540002-236c-4c94-8fa9-944a3e5353fa";
    const CURRENT_BAT: &'static str = "fc540007-236c-4c94-8fa9-944a3e5353fa";

    pub async fn find_mug(central: &Adapter) -> Result<Self> {
        let peripherals = central.peripherals().await?;
        for device in peripherals {
            println!("found device: {:?}", device.address());
            let properties = device.properties().await?;
            // println!("props: {properties:?}");
            let local_name = &properties
                .context("Failed to get properties from device")?
                .local_name;
            println!("name: {local_name:?}");
            if local_name.iter().any(|name| name.contains("Ember")) {
                println!("connecting");
                device.connect().await?;
                println!("discovering services");
                device.discover_services().await?;
                let mug = Mug { device };
                mug.check_characteristics()?;
                return Ok(mug);
            }
        }
        Err(anyhow!("Failed to find ble device with name Ember"))
    }
    pub async fn get_current_temp(&self) -> Result<TemperatureInterval> {
        let chars = self.device.characteristics();
        let cmd_char = chars
            .iter()
            .find(|c| c.uuid.to_string() == Self::CURRENT_TEMP)
            .unwrap();

        println!("{cmd_char}");

        let temp = self.device.read(cmd_char).await?;
        let temp: [u8; 2] = temp
            .try_into()
            .map_err(|temp| anyhow::anyhow!("Couldn't map {temp:?} to u64"))?;
        let temp = u16::from_le_bytes(temp);
        let temp = temp as f32 / 100.0;
        Ok(TemperatureInterval::new::<degree_celsius>(temp))
    }
    fn check_characteristics(&self) -> Result<()> {
        let chars = self
            .device
            .characteristics()
            .into_iter()
            .map(|char| char.uuid.to_string())
            .collect();
        let expected_chars = HashSet::from([
            String::from(Self::TARGET_TEMP),
            // String::from(Self::LED_COLOUR),
            String::from(Self::CURRENT_TEMP),
            String::from(Self::CURRENT_BAT),
        ]);
        if expected_chars.is_subset(&chars) {
            if !chars.is_subset(&expected_chars) {
                println!(
                    "Selected ble device: {} has extra unknown characteristics: {:?}",
                    self.device.address(),
                    expected_chars.difference(&chars),
                )
            }
            Ok(())
        } else {
            Err(anyhow!(
                "Selected ble device: {} does not support characteristics: {:?}",
                self.device.address(),
                expected_chars.difference(&chars),
            ))
        }
    }
}
