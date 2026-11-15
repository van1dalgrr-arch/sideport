use anyhow::{Context, Result, bail};
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

const SOCKET_PATH: &str = "/var/run/usbmuxd";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ListResponse {
    device_list: Vec<Device>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Device {
    #[serde(rename = "DeviceID")]
    pub device_id: u32,
    pub properties: Properties,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Properties {
    pub connection_type: String,
    pub serial_number: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ResultResponse {
    number: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PairRecordResponse {
    pair_record_data: plist::Data,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PairRecord {
    #[serde(rename = "HostID")]
    pub host_id: String,
    #[serde(rename = "SystemBUID")]
    pub system_buid: String,
    pub host_certificate: plist::Data,
    pub host_private_key: plist::Data,
    pub device_certificate: plist::Data,
}

pub struct Usbmux {
    stream: UnixStream,
}

impl Usbmux {
    pub async fn connect() -> Result<Self> {
        let stream = UnixStream::connect(SOCKET_PATH)
            .await
            .context("cannot connect to usbmuxd")?;
        Ok(Self { stream })
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

    pub async fn devices(&mut self) -> Result<Vec<Device>> {
        let request = new_request("ListDevices");

        self.send(&request).await?;
        let payload = self.recv().await?;

        let response: ListResponse = plist::from_bytes(&payload)?;

        Ok(response.device_list)
    }

    pub async fn connect_port(mut self, device_id: u32, port: u16) -> Result<UnixStream> {
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

    pub async fn read_pair_record(&mut self, udid: &str) -> Result<PairRecord> {
        let mut request = new_request("ReadPairRecord");
        request.insert("PairRecordID".into(), udid.into());

        self.send(&request).await?;
        let payload = self.recv().await?;

        let response: PairRecordResponse =
            plist::from_bytes(&payload).context("device is not paired with this Mac")?;
        let record: PairRecord = plist::from_bytes(response.pair_record_data.as_ref())?;

        Ok(record)
    }
}

fn new_request(message_type: &str) -> plist::Dictionary {
    let mut request = plist::Dictionary::new();
    request.insert("MessageType".into(), message_type.into());
    request.insert("ClientVersionString".into(), "sideport".into());
    request.insert("ProgName".into(), "sideport".into());
    request
}
