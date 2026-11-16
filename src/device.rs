use anyhow::{Context, Result};
use serde::Deserialize;

use crate::afc::Afc;
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
    let device = first_device().await?;

    let mut service = open_service(&device, "com.apple.mobile.installation_proxy").await?;

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

pub async fn open_stream(device: &Device, name: &str) -> Result<Box<dyn Stream>> {
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

    Ok(stream)
}

pub async fn open_service(device: &Device, name: &str) -> Result<Service> {
    Ok(Service::new(open_stream(device, name).await?))
}

async fn first_device() -> Result<Device> {
    let mut mux = Usbmux::connect().await?;
    let devices = mux.devices().await?;

    let usb = devices
        .iter()
        .position(|d| d.properties.connection_type == "USB")
        .unwrap_or(0);

    devices
        .into_iter()
        .nth(usb)
        .context("no device found")
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct InstallResponse {
    status: Option<String>,
    percent_complete: Option<u64>,
    error: Option<String>,
    error_description: Option<String>,
}

pub async fn install(path: &str) -> Result<()> {
    let device = first_device().await?;

    let name = std::path::Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .context("bad ipa path")?;
    let remote = format!("PublicStaging/{name}");

    let mut afc = Afc::new(open_stream(&device, "com.apple.afc").await?);
    afc.make_dir("PublicStaging").await?;
    afc.upload(path, &remote).await?;

    let mut service = open_service(&device, "com.apple.mobile.installation_proxy").await?;

    let mut request = plist::Dictionary::new();
    request.insert("Command".into(), "Install".into());
    request.insert("PackagePath".into(), remote.into());
    request.insert(
        "ClientOptions".into(),
        plist::Value::Dictionary(plist::Dictionary::new()),
    );

    service.send(&request).await?;

    loop {
        let payload = service.recv().await?;
        let response: InstallResponse = plist::from_bytes(&payload)?;

        if let Some(error) = response.error {
            anyhow::bail!(
                "install failed: {error}: {}",
                response.error_description.unwrap_or_default()
            );
        }

        let status = response.status.unwrap_or_default();
        if status == "Complete" {
            break;
        }

        println!("{}% {status}", response.percent_complete.unwrap_or(0));
    }

    println!("installed");

    Ok(())
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
