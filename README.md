# Quant-Voice-Vector

Настольное приложение, которое превращает записи голоса человека в файл голосового профиля
`.qvoice.json`. Файл загружается в профиль человека в
[Quant-Core](https://github.com/Quant-Ai-Assistant/Quant-Core), и ассистент узнаёт этого человека
по голосу.

Приложение ничего не знает о пользователях Quant-Core и не обращается к его API. Связь между
проектами — только файл.

## Важно

- **Голос — не пароль.** Запись или синтез голоса могут обмануть такую проверку. Узнавание по
  голосу годится для удобства («кто сейчас говорит»), но не для защиты доступа.
- **Файл профиля — биометрические данные.** Он создаётся только по явному действию человека и
  никуда не отправляется.
- **Обработка локальная.** Звук не покидает компьютер и не сохраняется на диск.
- **Что хранится.** Только сохранённые профили — файлы `.qvoice.json` в папке данных приложения. Черновик, который ещё набирает речь, живёт в памяти до закрытия окна.

## Установка

Установщики лежат на странице
[Releases](https://github.com/Quant-Ai-Assistant/Quant-Voice-Vector/releases).

| Система | Файл |
| --- | --- |
| macOS на Apple Silicon | `..._aarch64.dmg` |
| macOS на Intel | `..._x64.dmg` |
| Windows | `..._x64-setup.exe` |
| Linux | `.AppImage` или `.deb` |

Приложение пока не подписано сертификатом разработчика, поэтому система предупредит при первом
запуске:

- **macOS:** откройте приложение, затем «Системные настройки → Конфиденциальность и безопасность →
  Всё равно открыть».
- **Windows:** в окне SmartScreen нажмите «Подробнее → Выполнить в любом случае».

## Как это устроено

| Часть | Где | Что делает |
| --- | --- | --- |
| Ядро | `crates/qvv-core` | Чтение звука, удаление тишины, расчёт векторов, сборка и проверка профиля |
| Приложение | `app` | Окно на Tauri 2, запись с микрофона, команды для интерфейса |
| Интерфейс | `ui` | HTML, CSS и JavaScript без сборки |
| Формат файла | `schema/qvoice.v1.schema.json` | JSON Schema, по ней же файл проверяет Quant-Core |

Векторы считает [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) 1.13.8 моделью
`3dspeaker_speech_campplus_sv_zh_en_16k-common_advanced` (192 числа в векторе), тишину вырезает
Silero VAD. Идентификаторы и контрольные суммы моделей закреплены в
`crates/qvv-core/src/model.rs`.

### Что должен повторить Quant-Core

Векторы сравнимы, только если обе стороны считают их одинаково:

1. Та же модель с той же контрольной суммой и та же версия sherpa-onnx.
2. Звук приведён к моно 16 кГц, тишина вырезана Silero VAD.
3. После `acceptWaveform` вызывается `inputFinished()`; без него вектор немного другой.
4. Вектор нормализуется (L2), сравнение — косинусная близость.

## Разработка

Нужны Rust и cmake (для сборки libopus).

```bash
./scripts/fetch-models.sh
cargo run -p qvv-app
```

Сборка пакета для своей системы (на macOS только из него в Dock видно правильное имя):

```bash
cd app && npx @tauri-apps/cli@2 build
```

Релиз: тег вида `v0.1.0` запускает сборку установщиков для всех систем и создаёт черновик
релиза на GitHub.

Проверка ядра без окна:

```bash
cargo run --release -p qvv-core --example qvv -- enroll models Alexey alexey.qvoice.json запись1.m4a запись2.ogg
cargo run --release -p qvv-core --example qvv -- verify models alexey.qvoice.json новая.wav
```

## Лицензия

AGPL-3.0-or-later.

В установщики входят чужие компоненты под своими лицензиями:

| Компонент | Лицензия |
| --- | --- |
| Модель голоса [3D-Speaker CAM++](https://modelscope.cn/models/iic/speech_campplus_sv_zh_en_16k-common_advanced) | Apache-2.0 |
| [Silero VAD](https://github.com/snakers4/silero-vad) | MIT |
| [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) | Apache-2.0 |
| [ONNX Runtime](https://github.com/microsoft/onnxruntime) | MIT |
| [libopus](https://opus-codec.org/license/) | BSD |
| [Symphonia](https://github.com/pdeljanov/Symphonia) | MPL-2.0 |
| [Tauri](https://github.com/tauri-apps/tauri) | Apache-2.0 или MIT |
