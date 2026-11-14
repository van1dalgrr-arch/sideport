use anyhow::{Result, bail};
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::lockdown::Lockdown;

const LOCKDOWN_PORT: u16 = 62078;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ListResponse {
    device_list: Vec<DeviceEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DeviceEntry {
    #[serde(rename = "DeviceID")]
    device_id: u32,
    properties: Properties,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Properties {
    connection_type: String,
    serial_number: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ResultResponse {
    number: u32,
}

struct Usbmux {
    stream: UnixStream,
}

impl Usbmux {
    async fn connect() -> Result<Usbmux> {
        let stream = UnixStream::connect("/var/run/usbmuxd").await?;
        Ok(Usbmux { stream })
    }

    async fn send(&mut self, request: &plist::Dictionary) -> Result<()> {
        let mut body = Vec::new();
        plist::to_writer_xml(&mut body, request)?;

        let len = (16 + body.len()) as u32;
        self.stream.write_all(&len.to_le_bytes()).await?;
        self.stream.write_all(&1u32.to_le_bytes()).await?;
        self.stream.write_all(&8u32.to_le_bytes()).await?;
        self.stream.write_all(&1u32.to_le_bytes()).await?;
        self.stream.write_all(&body).await?;

        Ok(())
    }

    async fn recv(&mut self) -> Result<Vec<u8>> {
        let mut header = [0u8; 16];
        self.stream.read_exact(&mut header).await?;
        let total = u32::from_le_bytes(header[0..4].try_into()?) as usize;

        let mut payload = vec![0u8; total - 16];
        self.stream.read_exact(&mut payload).await?;

        Ok(payload)
    }

    async fn connect_port(mut self, device_id: u32, port: u16) -> Result<UnixStream> {
        let mut request = new_request("Connect");
        request.insert("DeviceID".into(), device_id.into());
        request.insert("PortNumber".into(), port.to_be().into());

        self.send(&request).await?;
        let payload = self.recv().await?;

        let response: ResultResponse = plist::from_bytes(&payload)?;
        if response.number != 0 {
            bail!("usbmuxd refused connection, code {}", response.number);
        }

        Ok(self.stream)
    }
}

fn new_request(message_type: &str) -> plist::Dictionary {
    let mut request = plist::Dictionary::new();
    request.insert("MessageType".into(), message_type.into());
    request.insert("ClientVersionString".into(), "sideport".into());
    request.insert("ProgName".into(), "sideport".into());
    request
}

pub async fn list() -> Result<()> {
    let mut mux = Usbmux::connect().await?;

    let request = new_request("ListDevices");

    mux.send(&request).await?;
    let payload = mux.recv().await?;

    let response: ListResponse = plist::from_bytes(&payload)?;

    if response.device_list.is_empty() {
        println!("no device found");
    }

    for d in &response.device_list {
        println!(
            "{} ({}), id {}",
            d.properties.serial_number, d.properties.connection_type, d.device_id
        );

        let conn = Usbmux::connect().await?;
        let stream = conn.connect_port(d.device_id, LOCKDOWN_PORT).await?;

        let mut lockdown = Lockdown::new(stream);
        lockdown.get_values().await?;
    }

    Ok(())
}
