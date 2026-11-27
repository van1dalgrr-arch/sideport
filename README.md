# sideport

[![CI](https://github.com/van1dalgrr-arch/sideport/actions/workflows/ci.yml/badge.svg)](https://github.com/van1dalgrr-arch/sideport/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-macOS-lightgrey.svg)](https://apple.com/macos)

**English** | [Русский](README.ru.md)

An open-source IPA sideloader for iPhone, written in Rust.

Install an IPA on your own iPhone with a free Apple ID, from the terminal, and re-sign it before the 7-day signature runs out. A free and open replacement for Sideloadly and AltStore.

## Features

- **Install** unsigned `.ipa` files on a connected iPhone over USB or Wi‑Fi
- **Refresh** an app: uninstall the old version and install the new one in one command
- **List** installed apps with name, version, and bundle identifier
- **Uninstall** apps by bundle identifier
- **Device management**: list connected iPhones with model and iOS version
- **Watch** usbmuxd connect/disconnect events in real time
- **IPA info**: read `CFBundleIdentifier` from any `.ipa`
- **Apple ID login** with two-factor authentication support
- **Certificate pinning**: TLS to the device is verified against the pairing record
- **No Apple libraries, no libimobiledevice** — pure Rust implementation of usbmuxd, lockdownd, AFC, and installation_proxy protocols

## Commands

| Command | Description |
|---------|-------------|
| `sideport devices` | List connected iPhones (USB and Wi‑Fi) |
| `sideport apps` | List user apps installed on the phone |
| `sideport install <file.ipa>` | Sign and install an IPA |
| `sideport refresh <file.ipa>` | Replace an installed app with a new IPA |
| `sideport uninstall <bundle id>` | Remove an app from the phone |
| `sideport login` | Sign in to your Apple ID |
| `sideport info <file.ipa>` | Print the bundle id of an IPA |
| `sideport watch` | Print device connect/disconnect events |
| `sideport --version` | Print version |
| `sideport --help` | Show usage |

## Quick start

```console
$ git clone https://github.com/van1dalgrr-arch/sideport
$ cd sideport
$ cargo install --path .
$ sideport devices
00008120-000A1B2C3D4E5F6A (USB), id 3
  My iPhone, iPhone15,4, iOS 26.2.1
  secure session ok, My iPhone

$ sideport login
Apple ID: you@example.com
Password:
2FA code: ******
logged in as Example User

$ sideport install UTM-SE.ipa
signing...
packing...
uploading 100%
5% CreatingStagingDirectory
...
installed
```

If `sideport devices` shows nothing, connect the iPhone with a cable, unlock it, and tap "Trust" on the device screen.

## How it works

sideport talks directly to `usbmuxd` over the Unix socket, with no Apple libraries and no `libimobiledevice`:

```
sideport ─ Unix socket ─▶ usbmuxd ─ USB / Wi‑Fi ─▶ iPhone
                                                    ├─ lockdownd           (port 62078)
                                                    ├─ AFC                 (file upload)
                                                    └─ installation_proxy  (install, list, remove)
```

1. **usbmuxd.** sideport connects to `/var/run/usbmuxd`, lists devices, reads the pairing record your Mac created when you tapped "Trust", and opens tunnels to ports on the phone.
2. **lockdownd.** Through a tunnel to port 62078 it starts a session with the host certificate from the pairing record and switches the connection to TLS.
3. **Services.** It asks lockdownd to start a service, gets a port, and opens a second TLS tunnel to it.
4. **AFC.** The IPA is uploaded to the phone over Apple File Conduit, a binary protocol with 40-byte packet headers.
5. **installation_proxy.** The phone unpacks, verifies, and installs the uploaded IPA and reports progress.

Signing uses the [isideload](https://crates.io/crates/isideload) crate: Apple ID login with two-factor authentication, the free development certificate, App IDs, provisioning profiles, and code signing.

Everything is async and built on [tokio](https://tokio.rs).

## Security

- **TLS to the phone is verified.** sideport pins the device certificate from the pairing record and refuses any other certificate. Certificate checks are never disabled.
- **Your password goes only to Apple.** You type it in the terminal; it is never stored or logged.
- **Anisette data comes from a third-party server.** Apple requires a device fingerprint ("anisette data") with every login. sideport gets it from the public server used by `isideload` (`ani.iloader.app`). The server does not receive your password. Generating it locally on macOS is on the roadmap.

Use a separate Apple ID for sideloading if you can.

## Build

You need macOS, an iPhone that trusts your Mac, and Rust 1.85 or newer.

```sh
git clone https://github.com/van1dalgrr-arch/sideport
cd sideport
cargo install --path .
sideport devices
```

### From source

```sh
cargo build --release
./target/release/sideport devices
```

### Development

```sh
cargo fmt          # format code
cargo clippy       # lint code
cargo test         # run unit tests
cargo build --release
```

## Roadmap

- [x] Read an IPA and parse `Info.plist`
- [x] List devices through `usbmuxd`, USB and Wi‑Fi
- [x] lockdownd session over TLS with certificate pinning
- [x] Start device services
- [x] List and remove installed apps
- [x] Upload an IPA over AFC and install it through `installation_proxy`
- [x] Device connect/disconnect events
- [x] Refresh: uninstall old version, install new one
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

Push notifications, iCloud, and Sign in with Apple usually do not work in apps signed with a free account.

## Disclaimer

sideport is not affiliated with Apple, Sideloadly, or AltStore. Install only apps that you have the right to use.

## Credits

- [isideload](https://github.com/nab138/isideload) by nab138: Apple ID login and signing
- [rustls](https://github.com/rustls/rustls), [tokio](https://tokio.rs), [plist](https://crates.io/crates/plist), [zip](https://crates.io/crates/zip)

## License

[MIT](LICENSE)
