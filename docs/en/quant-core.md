# Quant-Core integration

Русская версия: [docs/ru/quant-core.md](../ru/quant-core.md)

Quant Voice Vector and [Quant-Core](https://github.com/Quant-Ai-Assistant/Quant-Core) share
nothing but the file. The app knows nothing about Quant-Core users and never calls its API.

| Project | Does |
| --- | --- |
| Quant Voice Vector | Takes audio, computes vectors, produces `.qvoice.json` |
| Quant-Core | Accepts the file into a person's profile, identifies the speaker of every voice message |

Receiving and storing profiles on the Quant-Core side is not written yet. This page is the
specification for it.

## The main rule

Vectors from different models, or from the same model with different preprocessing, cannot be
compared. Both sides must compute a vector in exactly the same way.

## How this app computes a vector

1. **Decode** the audio to floating-point samples and mix it down to mono by averaging channels.
2. **Resample** to 16 kHz with sherpa-onnx's `LinearResampler`.
3. **Remove silence** with Silero VAD: threshold 0.5, minimum silence 0.5 s, minimum speech
   0.25 s, window 512 samples, maximum speech 20 s. The speech segments are concatenated.
4. **Split** the speech into 8-second chunks. A last chunk shorter than 3 seconds is dropped.
5. **Compute** a vector per chunk: create a stream, `acceptWaveform(16000, samples)`, then
   `inputFinished()`, then `compute`.
6. **Normalize** the vector to unit length (L2). The model's raw output is not normalized.

The pinned values are in [`crates/qvv-core/src/model.rs`](../../crates/qvv-core/src/model.rs)
and [`engine.rs`](../../crates/qvv-core/src/engine.rs):

| What | Value |
| --- | --- |
| Engine | sherpa-onnx 1.13.8 |
| Model | `3dspeaker_speech_campplus_sv_zh_en_16k-common_advanced.onnx` |
| Model SHA-256 | `aa3cfc16963a10586a9393f5035d6d6b57e98d358b347f80c2a30bf4f00ceba2` |
| Vector length | 192 |
| VAD | `silero_vad.onnx`, SHA-256 `9e2449e1087496d8d4caba907f23e0bd3f78d91fa552479bb9c23ac09cbb1fd6` |

## What Quant-Core has to do

- Use `sherpa-onnx-node` of the same version with the same model file, and check its SHA-256 at
  startup.
- Reject a profile whose `model.id` or `model.sha256` differs from its own.
- For an incoming voice message (Opus in WebM by default), repeat steps 1–3 and 5–6 on the whole
  message, then compare the vector with the profiles.
- **Call `inputFinished()` before `compute`.** Without it the last frames are not flushed, and
  the same audio gives a slightly different vector: cosine about 0.999 instead of 1.
- Choose the recognition threshold on its own data. This app only shows the similarity.

## What was measured

- **Rust and Node.js agree.** On 20 test recordings the vectors from the Rust crate and from
  `sherpa-onnx-node` matched to the sixth decimal place when the call sequence was the same.
- **Opus does not break it.** The same recording before and after Opus compression at 24 kbit/s
  scored 0.93–0.99 against itself.
- **Different microphones matter more than the codec.** The same person through two microphones
  scored about 0.5–0.8; a different person stayed below about 0.5 in our small test set.

The test set was small: five speakers in Chinese and English, and one male and one female voice
in Russian. A male voice against another male voice in Russian was not tested.

## When the model changes

Every existing profile becomes useless and has to be recorded again. Change the constants in
`model.rs`, the model in Quant-Core and this page in the same release.
