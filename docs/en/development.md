# Development

Русская версия: [docs/ru/development.md](../ru/development.md)

[`AGENTS.md`](../../AGENTS.md) holds the project rules and
[`CodeStyle.md`](../../CodeStyle.md) the mandatory standard. Read both first.

## Requirements

- **Rust**, stable toolchain, with `rustfmt` and `clippy`
- **cmake**, to build libopus
- **Node.js**, for the Tauri command-line tool and for syntax checks

macOS:

```sh
brew install rustup cmake node
rustup default stable
```

Debian and Ubuntu also need the WebKit and audio packages:

```sh
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libasound2-dev cmake
```

Windows needs the Visual Studio C++ build tools and cmake.

## Commands

```sh
./scripts/fetch-models.sh                  # download the models into models/, verify checksums
cargo run -p qvv-app                       # run the app from source
./scripts/check.sh                         # the quality gate
cd app && npx @tauri-apps/cli@2 build      # build an installer for this system
```

The core can be tried without the window:

```sh
cargo run --release -p qvv-core --example qvv -- enroll models Alexey alexey.qvoice.json one.m4a two.ogg
cargo run --release -p qvv-core --example qvv -- verify models alexey.qvoice.json new.wav
```

On macOS the Dock shows the right name and icon only for the built bundle, not for
`cargo run`.

## Layout

| Path | What it is |
| --- | --- |
| `crates/qvv-core` | The core: decoding, VAD, vectors, building and reading profiles. No window, no microphone |
| `crates/qvv-core/src/model.rs` | Pinned models and their checksums |
| `app` | The Tauri app: commands, microphone capture, icons, bundle configuration |
| `ui` | The interface: `index.html`, `style.css`, `main.js`, `texts.js` |
| `schema` | JSON Schema of the file format |
| `scripts` | `fetch-models.sh` and `check.sh` |
| `docs/en`, `docs/ru` | Guides in two languages |
| `models`, `samples` | Local only, ignored by git |

## Quality gate

`./scripts/check.sh` runs, in order:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `node --check` for every script in `ui/`
5. A parse of the schema and the Tauri configuration files

GitHub Actions runs the same script on every push to `main` and on every pull request. Nothing
is pushed while it fails.

The tests build vectors by hand, so they need neither models nor audio. Anything that touches
real audio, the microphone or the interface is tried by hand before a push.

## Interface texts

- Interface strings for both languages are in `STRINGS` in `ui/main.js`. A new string is added
  to both.
- Reading texts are in `ui/texts.js`, one paragraph or phrase per line. The app splits them into
  short lines by sentences and commas.

## Releases

1. Raise `version` in the root `Cargo.toml` and in the README badge.
2. Run `./scripts/check.sh`, commit and push.
3. Tag the commit: `git tag v0.2.0 && git push origin v0.2.0`.
4. GitHub Actions builds installers for macOS (Apple Silicon and Intel), Windows and Linux and
   attaches them to a draft release.
5. Install at least one of them, then publish the draft.

The installers are not signed with paid certificates. macOS builds get an ad-hoc signature so
that a downloaded app is not reported as damaged; signing with an Apple Developer ID and
notarization need the owner's certificate in the repository secrets.

## Changing the model or the format

Both are breaking changes for Quant-Core. Read
[Quant-Core integration](quant-core.md) and [File format](file-format.md) first, and change the
code, the schema and both projects together.
