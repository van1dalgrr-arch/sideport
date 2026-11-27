# sideport

[English](README.md) | **Русский**

[![CI](https://github.com/van1dalgrr-arch/sideport/actions/workflows/ci.yml/badge.svg)](https://github.com/van1dalgrr-arch/sideport/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/van1dalgrr-arch/sideport?include_prereleases)](https://github.com/van1dalgrr-arch/sideport/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-macOS-lightgrey.svg)](https://apple.com/macos)
[![Stars](https://img.shields.io/github/stars/van1dalgrr-arch/sideport?style=social)](https://github.com/van1dalgrr-arch/sideport/stargazers)

Открытый сайдлоадер IPA для iPhone, написанный на Rust.

Устанавливает IPA на твой iPhone с бесплатным Apple ID прямо из терминала и переподписывает приложение до того, как истечёт 7-дневная подпись. Бесплатная и открытая замена Sideloadly и AltStore.

---

## Почему sideport?

| | sideport | Sideloadly | AltStore |
|---|---|---|---|
| **Открытый исходный код** | ✅ MIT | ❌ Закрытый | ❌ Закрытый |
| **Без библиотек Apple** | ✅ Чистый Rust | ❌ Использует либы Apple | ❌ Использует либы Apple |
| **Без libimobiledevice** | ✅ Собственная реализация протоколов | ❌ Зависит от неё | ❌ Зависит от неё |
| **Терминал primero** | ✅ Да | ❌ Только GUI | ❌ Только GUI |
| **Проверка сертификата** | ✅ Верифицированный TLS | ⚠️ Базовая | ⚠️ Базовая |
| **Бесплатный Apple ID** | ✅ Да | ✅ Да | ✅ Да |
| **Обновление (re-sign)** | ✅ Одна команда | ⚠️ Вручную | ⚠️ Вручную |

sideport создан для разработчиков и пользователей, которые хотят полный контроль над процессом установки — нет проприетарных зависимостей, нет фоновых демонов, только чистый CLI, который общается напрямую с твоим iPhone.

---

## Возможности

- **Установка** подписанных `.ipa` на подключённый iPhone по USB или Wi‑Fi
- **Обновление приложения** (`refresh`): удаляет старую версию и ставит новую одной командой
- **Список приложений** с названием, версией и идентификатором пакета
- **Удаление** приложений по идентификатору пакета
- **Управление устройствами:** список подключённых iPhone с моделью и версией iOS
- **Наблюдение** за событиями подключения и отключения iPhone в реальном времени
- **Информация об IPA:** чтение `CFBundleIdentifier` из любого `.ipa`
- **Вход в Apple ID** с поддержкой двухфакторной аутентификации
- **Проверка сертификата:** TLS с телефона верифицируется по pairing record
- **Без библиотек Apple и без libimobiledevice** — чистая реализация протоколов usbmuxd, lockdownd, AFC и installation_proxy на Rust

---

## Команды

| Команда | Описание |
|---------|----------|
| `sideport devices` | Список подключённых iPhone (USB и Wi‑Fi) |
| `sideport apps` | Список установленных пользовательских приложений |
| `sideport install <file.ipa>` | Подписать и установить IPA |
| `sideport refresh <file.ipa>` | Заменить установленное приложение новым IPA |
| `sideport uninstall <bundle id>` | Удалить приложение с телефона |
| `sideport login` | Войти в Apple ID |
| `sideport info <file.ipa>` | Вывести bundle id из IPA |
| `sideport watch` | Вывести события подключения/отключения устройств |
| `sideport --version` | Вывести версию |
| `sideport --help` | Показать справку |

---

## Быстрый старт

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

$ sideport install MyApp.ipa
signing...
packing...
uploading 100%
5% CreatingStagingDirectory
...
installed
```

Если `sideport devices` ничего не показывает, подключи iPhone кабелем, разблокируй его и нажми «Доверять» на экране телефона.

---

## Установка

### Из исходников (cargo)

Нужны macOS, iPhone, который доверяет твоему Mac, и Rust 1.85 или новее.

```sh
git clone https://github.com/van1dalgrr-arch/sideport
cd sideport
cargo install --path .
sideport devices
```

### Из готовых бинарников

Предсобранные бинарники прикреплены к каждому релизу на GitHub:

```sh
# Скачать последний бинарник для macOS
curl -L https://github.com/van1dalgrr-arch/sideport/releases/latest/download/sideport-x86_64-apple-darwin -o sideport
chmod +x sideport
./sideport devices
```

### Homebrew (скоро)

```sh
# brew install van1dalgrr-arch/tap/sideport  # планируется
```

---

## Разработка

```sh
cargo fmt          # форматирование кода
cargo clippy       # линтинг кода (ошибки = предупреждения)
cargo test         # запуск юнит и интеграционных тестов
cargo build --release
```

---

## Как это устроено

sideport общается с `usbmuxd` через Unix-сокет, без библиотек Apple и без `libimobiledevice`:

```
sideport ─ Unix-сокет ─▶ usbmuxd ─ USB / Wi‑Fi ─▶ iPhone
                                                    ├─ lockdownd           (порт 62078)
                                                    ├─ AFC                 (заливка файлов)
                                                    └─ installation_proxy  (установка, список, удаление)
```

1. **usbmuxd.** sideport соединяется с `/var/run/usbmuxd`, получает список устройств, читает pairing record, который Mac создал при нажатии «Доверять», и открывает туннели к портам телефона.
2. **lockdownd.** Через туннель к порту 62078 начинает сессию с сертификатом из pairing record и переводит соединение в TLS.
3. **Сервисы.** Просит `lockdownd` запустить сервис, получает порт и открывает второй TLS-туннель к нему.
4. **AFC.** IPA заливается на телефон по Apple File Conduit — бинарному протоколу с 40-байтными заголовками пакетов.
5. **installation_proxy.** Телефон распаковывает, проверяет и устанавливает залитый IPA и сообщает прогресс.

Подпись сделана на крейте [isideload](https://crates.io/crates/isideload): вход в Apple ID с двухфакторной аутентификацией, бесплатный сертификат разработчика, App ID, provisioning-профили и подпись кода.

Весь код асинхронный и работает на [tokio](https://tokio.rs).

---

## Безопасность

- **TLS с телефоном проверяется.** sideport сверяет сертификат телефона с тем, что лежит в pairing record, и отвергает любой другой. Проверка сертификатов не отключается.
- **Пароль уходит только в Apple.** Ты вводишь его в терминале, он нигде не сохраняется и не пишется в логи.
- **Anisette-данные берутся со стороннего сервера.** Apple требует «отпечаток устройства» (anisette) при каждом входе. Сейчас sideport получает его с публичного сервера, который использует `isideload` (`ani.iloader.app`). Пароль этот сервер не получает. Генерация отпечатка прямо на Mac есть в плане.

Если можешь, используй для сайдлоада отдельный Apple ID.

---

## План

- [x] Чтение IPA и разбор `Info.plist`
- [x] Список устройств через `usbmuxd`, USB и Wi‑Fi
- [x] Сессия `lockdownd` по TLS с проверкой сертификата
- [x] Запуск сервисов телефона
- [x] Список и удаление установленных приложений
- [x] Заливка IPA по AFC и установка через `installation_proxy`
- [x] События подключения и отключения телефона
- [x] Обновление: удаление старой версии, установка новой
- [x] Автоотзыв старого сертификата при достижении лимита сертификатов
- [ ] Вход в Apple ID и подпись, проверенные от начала до конца
- [ ] Хранение сессии Apple ID в Keychain
- [ ] `refresh`: переподпись установленных приложений до истечения 7 дней
- [ ] Локальный anisette на macOS
- [ ] `--help` и понятные сообщения об ошибках
- [ ] Готовые бинарники и Homebrew
- [ ] Swift + SwiftUI приложение с графическим интерфейсом
- [ ] Веб-интерфейс (`sideportd`)

---

## Ограничения бесплатного Apple ID

Эти ограничения задаёт Apple, они одинаковы для всех сайдлоадеров:

- подпись действует 7 дней;
- одновременно на устройстве может быть до 3 установленных так приложений;
- до 10 новых App ID в неделю; каждому расширению приложения нужен свой App ID.

Пуш-уведомления, iCloud и вход через Apple в приложениях, подписанных бесплатным аккаунтом, обычно не работают.

---

## Как внести вклад

Вклады приветствуются! Пожалуйста, ознакомьтесь с правилами contributing перед отправкой pull request.

1. Форкните репозиторий
2. Создайте ветку функции (`git checkout -b feature/amazing-feature`)
3. Закоммитьте изменения (`git commit -m 'feat: добавить невероятную фичу'`)
4. Запушьте в ветку (`git push origin feature/amazing-feature`)
5. Откройте Pull Request

Пожалуйста, запустите `cargo fmt`, `cargo clippy -- -D warnings` и `cargo test` перед отправкой.

---

## Поддержка

- **Проблемы:** [GitHub Issues](https://github.com/van1dalgrr-arch/sideport/issues)
- **Обсуждения:** [GitHub Discussions](https://github.com/van1dalgrr-arch/sideport/discussions)
- **Безопасность:** Пожалуйста, раскрывайте уязвимости безопасности приватно через GitHub Security Advisories

---

## Отказ от ответственности

sideport не связан с Apple, Sideloadly и AltStore. Устанавливай только те приложения, на использование которых у тебя есть право.

---

## Благодарности

- [isideload](https://github.com/nab138/isideload) от nab138: вход в Apple ID и подпись
- [rustls](https://github.com/rustls/rustls), [tokio](https://tokio.rs), [plist](https://crates.io/crates/plist), [zip](https://crates.io/crates/zip)

---

## Лицензия

[MIT](LICENSE)

---

*Если sideport оказался полезен, поставь ⭐ звёзду репозиторию, чтобы другие могли его найти.*
