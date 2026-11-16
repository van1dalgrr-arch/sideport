use anyhow::Result;
use serde::Deserialize;

use crate::usbmux::{Properties, Usbmux};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Message {
    message_type: String,
    #[serde(rename = "DeviceID")]
    device_id: u32,
    properties: Option<Properties>,
}

enum Event {
    Attached {
        id: u32,
        udid: String,
        connection: String,
    },
    Detached {
        id: u32,
    },
}

pub async fn run() -> Result<()> {
    let mut mux = Usbmux::connect().await?;
    mux.listen().await?;

    loop {
        let payload = mux.next_message().await?;
        let message: Message = plist::from_bytes(&payload)?;

        match to_event(message) {
            Some(Event::Attached {
                id,
                udid,
                connection,
            }) => println!("+ {udid} ({connection}), id {id}"),
            Some(Event::Detached { id }) => println!("- id {id}"),
            None => {}
        }
    }
}

fn to_event(message: Message) -> Option<Event> {
    match message.message_type.as_str() {
        "Attached" => {
            let properties = message.properties?;

            Some(Event::Attached {
                id: message.device_id,
                udid: properties.serial_number,
                connection: properties.connection_type,
            })
        }
        "Detached" => Some(Event::Detached {
            id: message.device_id,
        }),
        _ => None,
    }
}
