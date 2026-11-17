# sideport

[English](README.md) | **Русский**

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Rust](https://img.shields.io/badge/rust-2024-orange.svg)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey.svg)

Открытый сайдлоадер IPA для iPhone, написанный на Rust.

Устанавливает IPA на твой iPhone с бесплатным Apple ID прямо из терминала и переподписывает приложение до того, как истечёт 7-дневная подпись. Бесплатная и открытая замена Sideloadly и AltStore.

> **Статус: в разработке.** Работа с телефоном проверена на настоящем iPhone: sideport заливает IPA и запускает установку. Вход в Apple ID и подпись написаны, но ещё не проверены от начала до конца, поэтому приложения пока не устанавливаются. План ниже.

## Команды

| Команда | Что делает |
|---|---|
| `sideport devices` | Показывает подключённые iPhone (USB и Wi‑Fi): имя, модель, версию iOS |
| `sideport apps` | Показывает пользовательские приложения на телефоне |
| `sideport install <file.ipa>` | Подписывает IPA твоим Apple ID, заливает и устанавливает |
| `sideport uninstall <bundle id>` | Удаляет приложение с телефона |
| `sideport watch` | Печатает события подключения и отключения iPhone |
| `sideport login` | Входит в Apple ID и печатает имя аккаунта |
| `sideport info <file.ipa>` | Печатает bundle id из IPA |

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

Если подключён кабель, sideport работает через него. Wi‑Fi используется, только когда кабеля нет.

## Как это устроено

Часть, которая общается с телефоном, написана с нуля, без библиотек Apple и без libimobiledevice:

```
sideport ─ Unix-сокет ─▶ usbmuxd ─ USB / Wi‑Fi ─▶ iPhone
                                                   ├─ lockdownd           (порт 62078)
                                                   ├─ AFC                 (заливка файлов)
                                                   └─ installation_proxy  (установка, список, удаление)
```

1. **usbmuxd.** sideport общается с демоном `usbmuxd` через `/var/run/usbmuxd`: получает список устройств, читает pairing record, который Mac создал, когда ты нажал «Доверять», и открывает туннели к портам телефона.
2. **lockdownd.** Через туннель к порту 62078 начинает сессию с сертификатом из pairing record и переводит соединение в TLS.
3. **Сервисы.** Просит `lockdownd` запустить сервис, получает порт и открывает к нему второй TLS-туннель.
4. **AFC.** IPA заливается на телефон по Apple File Conduit: небольшой бинарный протокол с 40-байтными заголовками пакетов.
5. **installation_proxy.** Телефон распаковывает, проверяет и устанавливает залитый IPA и сообщает прогресс.

Подпись сделана на крейте [isideload](https://crates.io/crates/isideload): вход в Apple ID с двухфакторной аутентификацией, бесплатный сертификат разработчика, App ID, provisioning-профили и подпись кода.

Весь код асинхронный и работает на [tokio](https://tokio.rs).

## Безопасность

- **TLS с телефоном проверяется.** sideport сверяет сертификат телефона с тем, что лежит в pairing record, и отвергает любой другой. Проверка сертификатов не отключается.
- **Пароль уходит только в Apple.** Ты вводишь его в терминале, он нигде не сохраняется и не пишется в логи.
- **Anisette-данные берутся со стороннего сервера.** Apple требует «отпечаток устройства» (anisette) при каждом входе. Сейчас sideport получает его с публичного сервера, который использует `isideload` (`ani.iloader.app`). Пароль этот сервер не получает. Генерация отпечатка прямо на Mac есть в плане.

Если можешь, используй для сайдлоада отдельный Apple ID.

## Сборка

Нужны macOS, iPhone, который доверяет твоему Mac, и Rust 1.85 или новее.

```sh
git clone https://github.com/van1dalgrr-arch/sideport
cd sideport
cargo install --path .
sideport devices
```

Если `sideport devices` ничего не показывает, подключи телефон кабелем, разблокируй его и нажми «Доверять».

## План

- [x] Чтение IPA и разбор `Info.plist`
- [x] Список устройств через `usbmuxd`, USB и Wi‑Fi
- [x] Сессия `lockdownd` по TLS с проверкой сертификата
- [x] Запуск сервисов телефона
- [x] Список и удаление установленных приложений
- [x] Заливка IPA по AFC и установка через `installation_proxy`
- [x] События подключения и отключения телефона
- [ ] Вход в Apple ID и подпись, проверенные от начала до конца
- [ ] Хранение сессии Apple ID в Keychain
- [ ] `refresh`: переподпись установленных приложений до истечения 7 дней
- [ ] Локальный anisette на macOS
- [ ] `--help` и понятные сообщения об ошибках
- [ ] Готовые бинарники и Homebrew
- [ ] Веб-интерфейс (`sideportd`)

## Ограничения бесплатного Apple ID

Эти ограничения задаёт Apple, они одинаковы для всех сайдлоадеров:

- подпись действует 7 дней;
- одновременно на устройстве может быть до 3 установленных так приложений;
- до 10 новых App ID в неделю; каждому расширению приложения нужен свой App ID.

Пуш-уведомления, iCloud и вход через Apple в приложениях, подписанных бесплатным аккаунтом, обычно не работают.

## Отказ от ответственности

sideport не связан с Apple, Sideloadly и AltStore. Устанавливай только те приложения, на использование которых у тебя есть право.

## Благодарности

- [isideload](https://github.com/nab138/isideload) от nab138: вход в Apple ID и подпись
- [rustls](https://github.com/rustls/rustls), [tokio](https://tokio.rs), [plist](https://crates.io/crates/plist), [zip](https://crates.io/crates/zip)

## Лицензия

[MIT](LICENSE)
