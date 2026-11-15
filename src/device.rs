use anyhow::Result;

use crate::lockdown::Lockdown;
use crate::usbmux::{Device, Usbmux};

const LOCKDOWN_PORT: u16 = 62078;

pub async fn open_lockdown(device: &Device) -> Result<Lockdown> {
    let mux = Usbmux::connect().await?;
    let stream = mux.connect_port(device.device_id, LOCKDOWN_PORT).await?;

    Ok(Lockdown::new(stream))
}

pub async fn open_session(device: &Device) -> Result<Lockdown> {
    let mut mux = Usbmux::connect().await?;
    let record = mux
        .read_pair_record(&device.properties.serial_number)
        .await?;

    let lockdown = open_lockdown(device).await?;

    lockdown.start_session(&record).await
}

pub async fn list() -> Result<()> {
    let mut mux = Usbmux::connect().await?;
    let devices = mux.devices().await?;

    if devices.is_empty() {
        println!("no device found");
    }

    for d in &devices {
        println!(
            "{} ({}), id {}",
            d.properties.serial_number, d.properties.connection_type, d.device_id
        );

        let mut lockdown = open_lockdown(d).await?;
        let info = lockdown.get_values().await?;
        println!(
            "  {}, {}, iOS {}",
            info.device_name, info.product_type, info.product_version
        );

        let mut session = open_session(d).await?;
        let info = session.get_values().await?;
        println!("  secure session ok, {}", info.device_name);
    }

    Ok(())
}
