use anyhow::{Context, Result};
use serde::Deserialize;

use crate::lockdown::Lockdown;
use crate::service::Service;
use crate::tls::{self, Stream};
use crate::usbmux::{Device, Usbmux};

const LOCKDOWN_PORT: u16 = 62078;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BrowseResponse {
    status: String,
    #[serde(default)]
    current_list: Vec<App>,
}

#[derive(Debug, Deserialize)]
struct App {
    #[serde(rename = "CFBundleIdentifier")]
    bundle_id: String,
    #[serde(rename = "CFBundleDisplayName")]
    name: Option<String>,
    #[serde(rename = "CFBundleShortVersionString")]
    version: Option<String>,
}

pub async fn apps() -> Result<()> {
    let mut mux = Usbmux::connect().await?;
    let devices = mux.devices().await?;
    let device = devices.first().context("no device found")?;

    let mut service = open_service(device, "com.apple.mobile.installation_proxy").await?;

    let attributes: Vec<plist::Value> = vec![
        "CFBundleIdentifier".into(),
        "CFBundleDisplayName".into(),
        "CFBundleShortVersionString".into(),
    ];

    let mut options = plist::Dictionary::new();
    options.insert("ApplicationType".into(), "User".into());
    options.insert("ReturnAttributes".into(), plist::Value::Array(attributes));

    let mut request = plist::Dictionary::new();
    request.insert("Command".into(), "Browse".into());
    request.insert("ClientOptions".into(), plist::Value::Dictionary(options));

    service.send(&request).await?;

    loop {
        let payload = service.recv().await?;
        let response: BrowseResponse = plist::from_bytes(&payload)?;

        for app in &response.current_list {
            println!(
                "{} {} ({})",
                app.name.as_deref().unwrap_or("?"),
                app.version.as_deref().unwrap_or("?"),
                app.bundle_id
            );
        }

        if response.status == "Complete" {
            break;
        }
    }

    Ok(())
}

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

pub async fn open_service(device: &Device, name: &str) -> Result<Service> {
    let mut mux = Usbmux::connect().await?;
    let record = mux
        .read_pair_record(&device.properties.serial_number)
        .await?;

    let lockdown = open_lockdown(device).await?;
    let mut session = lockdown.start_session(&record).await?;
    let info = session.start_service(name).await?;

    let mux = Usbmux::connect().await?;
    let stream = mux.connect_port(device.device_id, info.port).await?;
    let mut stream: Box<dyn Stream> = Box::new(stream);

    if info.enable_service_ssl {
        stream = tls::connect(stream, &record).await?;
    }

    Ok(Service::new(stream))
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
