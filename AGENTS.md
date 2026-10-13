# Quant-Voice-Vector repository

Desktop app that turns voice recordings into a voice profile file, `.qvoice.json`, for
[Quant-Core](https://github.com/Quant-Ai-Assistant/Quant-Core). These rules apply to human
developers and AI agents alike. [CLAUDE.md](CLAUDE.md) points here.

## Read first

- [CodeStyle.md](CodeStyle.md) is the mandatory standard. Read it before any implementation, fix
  or refactor.
- [docs/en/development.md](docs/en/development.md) explains the layout, the commands and the
  release process.
- [docs/en/quant-core.md](docs/en/quant-core.md) is the contract with Quant-Core.

## Rules that must not be broken

1. **One model on both sides.** Vectors from different models, or from different preprocessing,
   cannot be compared. Anything that changes a vector (the model file, the sherpa-onnx version,
   resampling, VAD settings, the `input_finished()` call, normalization) is a breaking change:
   it invalidates every existing profile and has to be mirrored in Quant-Core.
2. **The file format is a shared contract.** Change `.qvoice.json` only together with
   `schema/qvoice.v1.schema.json`, the format guide and Quant-Core. Incompatible changes raise
   `version`.
3. **Voice data is biometric.** Never commit `*.qvoice.json` or audio. Never write raw audio to
   disk. Never send audio or profiles anywhere: all processing stays on the device.
4. **A voice is not a password.** Do not present the similarity check as authentication, in code,
   interface texts or documentation.
5. **Code is written from scratch.** Do not copy code from other projects. The license is
   AGPL-3.0-or-later; new dependencies need a compatible license and the owner's agreement.

## Git workflow

- Commit and push only when the owner asks.
- Commits use the owner's GitHub noreply address, never a personal email.
- Commit only files that belong to the task. Never force-push.
- A release is a tag like `v0.1.0`; it builds installers and creates a draft release. Bump the
  version in `Cargo.toml` first.

## Quality gate

`./scripts/check.sh` has to pass before every push: formatting, Clippy with warnings as errors,
tests, and syntax checks for the interface scripts and JSON files. A push is not allowed when a
check fails or could not be run; say so instead of skipping it.

For documentation-only changes, check the content, the links and that the English and Russian
versions say the same thing.

## Languages

- Code, comments, commit messages and `README.md` are in English.
- Every guide exists in English (`docs/en`) and Russian (`docs/ru`). Update both in the same
  change.
- Interface texts live in `ui/main.js` (`STRINGS`) for both languages; the reading texts live in
  `ui/texts.js`.
