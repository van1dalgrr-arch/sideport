# sideport

**English** | [Русский](README.ru.md)

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Rust](https://img.shields.io/badge/rust-2024-orange.svg)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey.svg)

An open-source IPA sideloader for iPhone, written in Rust.

Install an IPA on your own iPhone with a free Apple ID, from the terminal, and re-sign it before the 7-day signature runs out. A free and open replacement for Sideloadly and AltStore.

> **Status: work in progress.** The device side works on a real iPhone: sideport uploads an IPA and starts the installation. Apple ID login and signing are written but not yet verified end to end, so apps do not install yet. See the roadmap below.

## Commands

| Command | What it does |
|---|---|
| `sideport devices` | Lists connected iPhones (USB and Wi‑Fi) with name, model and iOS version |
| `sideport apps` | Lists user apps installed on the phone |
| `sideport install <file.ipa>` | Signs the IPA with your Apple ID, uploads it and installs it |
| `sideport uninstall <bundle id>` | Removes an app from the phone |
| `sideport watch` | Prints events when an iPhone is connected or disconnected |
| `sideport login` | Signs in to your Apple ID and prints the account name |
| `sideport info <file.ipa>` | Prints the bundle id of an IPA |

```console
$ sideport devices
00008120-000A1B2C3D4E5F6A (USB), id 3
  My iPhone, iPhone15,4, iOS 26.2.1
  secure session ok, My iPhone

$ sideport apps
Telegram 12.1 (ph.telegra.Telegraph)
UTM SE 5.0.9 (com.utmapp.UTM-SE)

$ sideport install UTM-SE.ipa
Apple ID: you@example.com
Password:
2FA code: ******
signing...
packing...
uploading 100%
5% CreatingStagingDirectory
...
installed
```

When a USB cable is connected, sideport uses it. Wi‑Fi is used only when there is no cable.

## How it works

The device side is written from scratch, with no Apple libraries and no libimobiledevice:

```
sideport ─ Unix socket ─▶ usbmuxd ─ USB / Wi‑Fi ─▶ iPhone
                                                    ├─ lockdownd           (port 62078)
                                                    ├─ AFC                 (file upload)
                                                    └─ installation_proxy  (install, list, remove)
```

1. **usbmuxd.** sideport talks to the `usbmuxd` daemon over `/var/run/usbmuxd`: it lists devices, reads the pairing record your Mac created when you tapped "Trust", and opens tunnels to ports on the phone.
2. **lockdownd.** Through a tunnel to port 62078 it starts a session with the host certificate from the pairing record and switches the connection to TLS.
3. **Services.** It asks `lockdownd` to start a service, gets a port and opens a second TLS tunnel to it.
4. **AFC.** The IPA is uploaded to the phone over Apple File Conduit, a small binary protocol with 40-byte packet headers.
5. **installation_proxy.** The phone unpacks, verifies and installs the uploaded IPA and reports progress.

Signing uses the [isideload](https://crates.io/crates/isideload) crate: Apple ID login with two-factor authentication, the free development certificate, App IDs, provisioning profiles and code signing.

Everything is async and built on [tokio](https://tokio.rs).

## Security

- **TLS to the phone is verified.** sideport pins the device certificate from the pairing record and refuses any other certificate. It does not turn certificate checks off.
- **Your password goes only to Apple.** You type it in the terminal; it is never stored or logged.
- **Anisette data comes from a third-party server.** Apple requires a device fingerprint ("anisette data") with every login. sideport currently gets it from the public server used by `isideload` (`ani.iloader.app`). The server does not receive your password. Generating it locally on the Mac is on the roadmap.

Use a separate Apple ID for sideloading if you can.

## Build

You need macOS, an iPhone that trusts your Mac, and Rust 1.85 or newer.

```sh
git clone https://github.com/van1dalgrr-arch/sideport
cd sideport
cargo install --path .
sideport devices
```

If `sideport devices` shows nothing, connect the phone with a cable, unlock it and tap "Trust".

## Roadmap

- [x] Read an IPA and parse `Info.plist`
- [x] List devices through `usbmuxd`, USB and Wi‑Fi
- [x] `lockdownd` session over TLS with certificate pinning
- [x] Start device services
- [x] List and remove installed apps
- [x] Upload an IPA over AFC and install it through `installation_proxy`
- [x] Device connect/disconnect events
- [ ] Apple ID login and signing, verified end to end
- [ ] Keep the Apple ID session in the Keychain
- [ ] `refresh`: re-sign installed apps before the 7 days run out
- [ ] Local anisette on macOS
- [ ] `--help` and better error messages
- [ ] Prebuilt binaries and Homebrew
- [ ] Web UI (`sideportd`)

## Free Apple ID limits

These limits come from Apple and apply to every sideloading tool:

- a signature is valid for 7 days;
- up to 3 sideloaded apps on a device at the same time;
- up to 10 new App IDs per week; every app extension needs its own App ID.

Push notifications, iCloud and Sign in with Apple usually do not work in apps signed with a free account.

## Disclaimer

sideport is not affiliated with Apple, Sideloadly or AltStore. Install only apps that you have the right to use.

## Credits

- [isideload](https://github.com/nab138/isideload) by nab138: Apple ID login and signing
- [rustls](https://github.com/rustls/rustls), [tokio](https://tokio.rs), [plist](https://crates.io/crates/plist), [zip](https://crates.io/crates/zip)

## License

[MIT](LICENSE)
