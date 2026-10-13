# Разработка

English version: [docs/en/development.md](../en/development.md)

Правила проекта записаны в [`AGENTS.md`](../../AGENTS.md), обязательный стандарт кода — в
[`CodeStyle.md`](../../CodeStyle.md). Сначала прочитайте оба.

## Что нужно

- **Rust**, стабильная версия, с `rustfmt` и `clippy`
- **cmake**, для сборки libopus
- **Node.js**, для командной утилиты Tauri и проверки синтаксиса

macOS:

```sh
brew install rustup cmake node
rustup default stable
```

В Debian и Ubuntu дополнительно нужны пакеты WebKit и звука:

```sh
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libasound2-dev cmake
```

В Windows нужны средства сборки C++ из Visual Studio и cmake.

## Команды

```sh
./scripts/fetch-models.sh                  # скачать модели в models/ и проверить контрольные суммы
cargo run -p qvv-app                       # запустить приложение из исходников
./scripts/check.sh                         # проверка качества
cd app && npx @tauri-apps/cli@2 build      # собрать установщик для этой системы
```

Ядро можно попробовать без окна:

```sh
cargo run --release -p qvv-core --example qvv -- enroll models Alexey alexey.qvoice.json one.m4a two.ogg
cargo run --release -p qvv-core --example qvv -- verify models alexey.qvoice.json new.wav
```

В macOS правильные имя и иконка в Dock видны только у собранного пакета, а не при `cargo run`.

## Устройство

| Путь | Что это |
| --- | --- |
| `crates/qvv-core` | Ядро: декодирование, VAD, векторы, сборка и чтение профилей. Без окна и микрофона |
| `crates/qvv-core/src/model.rs` | Закреплённые модели и их контрольные суммы |
| `app` | Приложение на Tauri: команды, запись с микрофона, иконки, настройки сборки |
| `ui` | Интерфейс: `index.html`, `style.css`, `main.js`, `texts.js` |
| `schema` | JSON Schema формата файла |
| `scripts` | `fetch-models.sh` и `check.sh` |
| `docs/en`, `docs/ru` | Руководства на двух языках |
| `models`, `samples` | Только локально, git их игнорирует |

## Проверка качества

`./scripts/check.sh` выполняет по порядку:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `node --check` для каждого скрипта в `ui/`
5. Разбор схемы и файлов настроек Tauri

GitHub Actions запускает тот же скрипт при каждой отправке в `main` и для каждого пулреквеста.
Пока он не проходит, ничего не отправляется.

Тесты строят векторы вручную, поэтому им не нужны ни модели, ни звук. Всё, что касается
настоящего звука, микрофона или интерфейса, перед отправкой проверяется руками.

## Тексты интерфейса

- Строки интерфейса для обоих языков находятся в `STRINGS` в `ui/main.js`. Новая строка
  добавляется в оба языка.
- Тексты для чтения находятся в `ui/texts.js`, по одному абзацу или фразе на строку. Приложение
  само делит их на короткие строки по предложениям и запятым.

## Релизы

1. Поднимите `version` в корневом `Cargo.toml` и в значке в README.
2. Запустите `./scripts/check.sh`, закоммитьте и отправьте.
3. Поставьте тег: `git tag v0.2.0 && git push origin v0.2.0`.
4. GitHub Actions соберёт установщики для macOS (Apple Silicon и Intel), Windows и Linux и
   приложит их к черновику релиза.
5. Установите хотя бы один из них, затем опубликуйте черновик.

Установщики не подписаны платными сертификатами. Сборки для macOS получают подпись без
сертификата, чтобы скачанное приложение не считалось повреждённым; подпись Apple Developer ID и
нотаризация требуют сертификата владельца в секретах репозитория.

## Смена модели или формата

И то и другое ломает совместимость с Quant-Core. Сначала прочитайте
[Связь с Quant-Core](quant-core.md) и [Формат файла](file-format.md), а код, схему и оба
проекта меняйте вместе.
