use anyhow::Result;
use serde::Deserialize;

use crate::usbmux::{Properties, Usbmux};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct Message {
    message_type: String,
    #[serde(rename = "DeviceID")]
    device_id: u32,
    properties: Option<Properties>,
}

pub async fn run() -> Result<()> {
    let mut mux = Usbmux::connect().await?;
    mux.listen().await?;

    loop {
        let payload = mux.next_message().await?;
        let message: Message = plist::from_bytes(&payload)?;
        println!("{message:?}");
    }
}
