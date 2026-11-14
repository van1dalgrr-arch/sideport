use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

pub async fn list() -> Result<()> {
    let mut stream = UnixStream::connect("/var/run/usbmuxd").await?;
    let mut request = plist::Dictionary::new();
    request.insert("MessageType".into(), "ListDevices".into());
    request.insert("ClientVersionString".into(), "ipaload".into());
    request.insert("ProgName".into(), "ipaload".into());

    let mut body = Vec::new();
    plist::to_writer_xml(&mut body, &request)?;

    let len = (16 + body.len()) as u32;
    stream.write_all(&len.to_le_bytes()).await?;
    stream.write_all(&1u32.to_le_bytes()).await?;
    stream.write_all(&8u32.to_le_bytes()).await?;
    stream.write_all(&1u32.to_le_bytes()).await?;
    stream.write_all(&body).await?;

    let mut header = [0u8; 16];
    stream.read_exact(&mut header).await?;
    let total = u32::from_le_bytes(header[0..4].try_into()?) as usize;

    let mut payload = vec![0u8; total - 16];
    stream.read_exact(&mut payload).await?;
    println!("{}", String::from_utf8_lossy(&payload));

    Ok(())
}
