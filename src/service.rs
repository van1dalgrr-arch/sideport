use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::tls::Stream;

pub struct Service {
    stream: Box<dyn Stream>,
}

impl Service {
    pub fn new(stream: Box<dyn Stream>) -> Self {
        Self { stream }
    }

    pub async fn send(&mut self, request: &plist::Dictionary) -> Result<()> {
        let mut body = Vec::new();
        plist::to_writer_xml(&mut body, request)?;

        let len = body.len() as u32;
        self.stream.write_all(&len.to_be_bytes()).await?;
        self.stream.write_all(&body).await?;
        self.stream.flush().await?;

        Ok(())
    }

    pub async fn recv(&mut self) -> Result<Vec<u8>> {
        let len = self.stream.read_u32().await?;

        let mut payload = vec![0u8; len as usize];
        self.stream.read_exact(&mut payload).await?;

        Ok(payload)
    }
}
