# Code Style — Quant-Voice-Vector

Mandatory for human developers and AI agents. Cite rule numbers in reviews.

## 0. General principles

- **0.1 One style.** New code reads like the code around it: same naming, same comment density,
  same idioms.
- **0.2 Simplicity.** The shortest correct solution wins. No abstraction without a second user,
  no configuration nobody asked for.
- **0.3 Automation decides formatting.** `cargo fmt` and Clippy are the authority; do not argue
  with them in review, and do not silence a lint without a comment that says why.
- **0.4 Comments explain why.** A comment states a reason, a constraint or a trap. It does not
  repeat the code.

## 1. Rust

- **1.1 Layers.** `crates/qvv-core` knows nothing about Tauri, windows or the microphone. `app`
  holds no audio or vector logic: it passes data to the core.
- **1.2 Pinned constants.** Everything that affects a vector lives in `model.rs` and `engine.rs`
  as a named constant with a comment. No magic numbers elsewhere.
- **1.3 Errors.** The core returns `qvv_core::Result` with a typed `Error`. No `unwrap()` or
  `expect()` on anything that depends on input, files or devices. `unwrap()` on a poisoned mutex
  and `expect()` on a programming invariant are allowed.
- **1.4 No `unsafe`.** If a dependency forces it, isolate it in one function with a `SAFETY:`
  comment.
- **1.5 Locks.** Never hold a mutex across slow work (decoding, VAD, embeddings). Compute first,
  lock to store the result.
- **1.6 Commands.** A Tauri command is thin: validate, call the core, return a serializable view.
  Slow commands are `#[tauri::command(async)]` so the window never freezes.
- **1.7 Audio stays in memory.** No function writes samples to disk or to a log.
- **1.8 Naming.** Types are nouns, functions are verbs, booleans read as facts (`can_finish`,
  `ready`). Units go in the name when they are not obvious (`speech_seconds`).

## 2. Interface (HTML, CSS, JavaScript)

- **2.1 No build step.** Plain files in `ui/`, no framework, no bundler, no CDN. The app works
  offline.
- **2.2 Text through `STRINGS`.** Every interface string exists in both languages; none is
  written in HTML. Error messages that come from the core are English-only for now.
- **2.3 Never `innerHTML` with data.** Names and file paths go in with `textContent`.
- **2.4 State, then render.** Handlers change state and call `render()`; they do not patch the
  DOM in scattered places.
- **2.5 Colors are tokens.** Use the CSS variables in `:root`; every color has a light and a dark
  value. Orange is for the main action only.
- **2.6 Content Security Policy stays strict.** No inline scripts, no inline event handlers, no
  remote resources.

## 3. Tests

- **3.1 Core logic has tests.** Profile building, outlier filtering, file reading and every rule
  of the format are covered by unit tests in the core.
- **3.2 Tests need no models and no audio.** They build vectors by hand, so they run anywhere in
  under a second.
- **3.3 A bug fix comes with a test** that fails without the fix.
- **3.4 Test names state the behavior:** `file_made_by_another_model_is_refused`.

## 4. Data and privacy

- **4.1** No profile, recording or model file is committed. `.gitignore` covers them; check
  `git status` before committing.
- **4.2** No network calls from the app. Models are bundled at build time.
- **4.3** Deleting a profile deletes its file.

## 5. Documentation

- **5.1** `README.md` is in English. Guides exist in `docs/en` and `docs/ru` with the same file
  names and the same content.
- **5.2** A change in behavior updates the guide in both languages in the same commit.
- **5.3** Documentation states facts about this version. Plans and unverified claims are marked
  as such.

## 6. Before a push

- **6.1** `./scripts/check.sh` passes.
- **6.2** The app was started and the changed path was tried by hand when the change touches the
  interface or audio.
- **6.3** Nothing in rules 4.1–4.3 was broken.
