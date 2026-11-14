# sideport

**English** | [Русский](README.ru.md)

An open-source IPA sideloader for iPhone, written in Rust.

The goal is a free replacement for Sideloadly: install an IPA on your own iPhone with a free Apple ID and re-sign it automatically before the 7-day signature expires.

> **Status: early work in progress.** sideport cannot install apps yet. Right now it reads IPA files and talks to a connected iPhone. See the roadmap below.

## What works today

| Command | What it does |
|---|---|
| `sideport info <file.ipa>` | Opens the IPA, finds `Info.plist` and prints the bundle id |
| `sideport devices` | Lists connected iPhones and prints the device properties returned by `lockdownd` |

```console
$ sideport info app.ipa
bundle id: com.example.app

$ sideport devices
00008120-000A1B2C3D4E5F6A (USB), id 1
<?xml version="1.0" encoding="UTF-8"?>
...
```

## How it works

sideport speaks Apple's device protocols directly, with no Apple libraries and no libimobiledevice:

1. It connects to the `usbmuxd` daemon over the Unix socket `/var/run/usbmuxd` and asks for the device list.
2. It asks `usbmuxd` to open a tunnel to port 62078 on the phone, where the `lockdownd` service listens.
3. It sends plist requests through that tunnel and reads the answers.

Everything is async and built on [tokio](https://tokio.rs).

## Build

You need macOS and a stable Rust toolchain (1.85 or newer).

```sh
git clone https://github.com/van1dalgrr-arch/sideport
cd sideport
cargo build --release
./target/release/sideport devices
```

Linux with a running `usbmuxd` should work too, but it has not been tested.

## Roadmap

- [x] Read an IPA and parse `Info.plist`
- [x] List devices through `usbmuxd`
- [x] Connect to `lockdownd` and query device values
- [ ] Human-readable device output (name, model, iOS version)
- [ ] Pairing and TLS session with the device
- [ ] Upload an IPA over AFC and install it through `installation_proxy`
- [ ] Apple ID login, certificates and provisioning profiles
- [ ] Re-sign an IPA
- [ ] `refresh`: re-sign installed apps before the 7 days run out
- [ ] GUI

## Free Apple ID limits

These limits come from Apple and apply to every sideloading tool:

- a signature is valid for 7 days;
- up to 3 sideloaded apps on a device at the same time;
- up to 10 new App IDs per week.

## Disclaimer

sideport is not affiliated with Apple or Sideloadly. Install only apps that you have the right to use.

## License

[MIT](LICENSE)
