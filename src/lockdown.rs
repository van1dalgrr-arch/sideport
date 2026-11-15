use crate::tls::{self, Stream};
use crate::usbmux::PairRecord;
use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StartSessionResponse {
    #[serde(rename = "SessionID")]
    session_id: String,
    #[serde(rename = "EnableSessionSSL")]
    enable_session_ssl: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeviceInfo {
    pub device_name: String,
    pub product_type: String,
    pub product_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GetValueResponse {
    value: DeviceInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StartServiceResponse {
    pub port: u16,
    #[serde(rename = "EnableServiceSSL", default)]
    pub enable_service_ssl: bool,
}

pub struct Lockdown {
    stream: Box<dyn Stream>,
}

impl Lockdown {
    pub async fn start_service(&mut self, name: &str) -> Result<StartServiceResponse> {
        let mut request = plist::Dictionary::new();
        request.insert("Label".into(), "sideport".into());
        request.insert("Request".into(), "StartService".into());
        request.insert("Service".into(), name.into());

        self.send(&request).await?;
        let payload = self.recv().await?;

        let response: StartServiceResponse =
            plist::from_bytes(&payload).context("device refused to start the service")?;

        Ok(response)
    }
    pub async fn start_session(mut self, record: &PairRecord) -> Result<Self> {
        let mut request = plist::Dictionary::new();
        request.insert("Label".into(), "sideport".into());
        request.insert("Request".into(), "StartSession".into());
        request.insert("HostID".into(), record.host_id.clone().into());
        request.insert("SystemBUID".into(), record.system_buid.clone().into());

        self.send(&request).await?;
        let payload = self.recv().await?;
        let response: StartSessionResponse =
            plist::from_bytes(&payload).context("lockdownd refused to start a session")?;

        if !response.enable_session_ssl {
            return Ok(self);
        }

        let stream = tls::connect(self.stream, record).await?;

        Ok(Self { stream })
    }
    pub fn new(stream: UnixStream) -> Self {
        Self {
            stream: Box::new(stream),
        }
    }

    async fn send(&mut self, request: &plist::Dictionary) -> Result<()> {
        let mut body = Vec::new();
        plist::to_writer_xml(&mut body, request)?;

        let len = body.len() as u32;
        self.stream.write_all(&len.to_be_bytes()).await?;
        self.stream.write_all(&body).await?;

        Ok(())
    }

    async fn recv(&mut self) -> Result<Vec<u8>> {
        let len = self.stream.read_u32().await?;

        let mut payload = vec![0u8; len as usize];
        self.stream.read_exact(&mut payload).await?;

        Ok(payload)
    }

    pub async fn get_values(&mut self) -> Result<DeviceInfo> {
        let mut request = plist::Dictionary::new();
        request.insert("Label".into(), "sideport".into());
        request.insert("Request".into(), "GetValue".into());

        self.send(&request).await?;
        let payload = self.recv().await?;

        let response: GetValueResponse = plist::from_bytes(&payload)?;

        Ok(response.value)
    }
}
