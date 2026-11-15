use anyhow::Result;
use serde::Deserialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

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

pub struct Lockdown {
    stream: UnixStream,
}

impl Lockdown {
    pub fn new(stream: UnixStream) -> Self {
        Self { stream }
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
