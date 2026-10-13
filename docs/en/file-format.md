# File format: `.qvoice.json`, version 1

Русская версия: [docs/ru/file-format.md](../ru/file-format.md)

A voice profile is a plain JSON file in UTF-8. There is no common standard for speaker profiles,
so the format is our own. It is described by
[`schema/qvoice.v1.schema.json`](../../schema/qvoice.v1.schema.json), and Quant-Core validates
incoming files against the same schema.

The format is a contract between two projects. It changes only together with Quant-Core, and an
incompatible change raises `version`.

## Example

Vectors are cut to two numbers here; a real file has exactly `model.dim` numbers in each.

```json
{
  "format": "quant-voiceprint",
  "version": 1,
  "profile": {
    "id": "0b6f1c2e-5d0a-4a7e-9c55-3f1f6f0c2a11",
    "name": "Alexey",
    "createdAt": "2026-10-13T18:00:00Z"
  },
  "model": {
    "id": "3dspeaker_speech_campplus_sv_zh_en_16k-common_advanced",
    "sha256": "aa3cfc16963a10586a9393f5035d6d6b57e98d358b347f80c2a30bf4f00ceba2",
    "dim": 192,
    "sampleRate": 16000
  },
  "normalized": true,
  "centroid": [0.0123, -0.0456],
  "embeddings": [
    { "vector": [0.0119, -0.0461], "speechSeconds": 8.0, "source": "microphone" },
    { "vector": [0.0131, -0.0448], "speechSeconds": 8.0, "source": "file" }
  ]
}
```

## Fields

| Field | Type | Meaning |
| --- | --- | --- |
| `format` | string | Always `quant-voiceprint` |
| `version` | integer | Format version, `1` |
| `profile.id` | string | UUID of the profile, generated when it is saved |
| `profile.name` | string | Display name, 1–200 characters |
| `profile.createdAt` | string | Creation time in UTC, RFC 3339 |
| `model.id` | string | Identifier of the model that computed the vectors |
| `model.sha256` | string | SHA-256 of the model's ONNX file, lowercase hex |
| `model.dim` | integer | Length of every vector |
| `model.sampleRate` | integer | Sample rate the audio was converted to, in Hz |
| `normalized` | boolean | Always `true`: vectors have unit length (L2) |
| `centroid` | number[] | Average of the vectors, normalized again; for a fast comparison |
| `embeddings` | object[] | Vectors of separate speech segments, 5 to 20 |
| `embeddings[].vector` | number[] | Vector of one segment |
| `embeddings[].speechSeconds` | number | Length of clean speech in the segment |
| `embeddings[].source` | string | `microphone` or `file` |

Unknown fields are not allowed at any level.

## What a reader must check

The schema covers types and ranges. Three rules are beyond it:

1. **Model.** `model.id` and `model.sha256` equal the reader's own model. A file made by another
   model is rejected: its vectors cannot be compared with anything.
2. **Length.** `centroid` and every `embeddings[].vector` have exactly `model.dim` numbers.
3. **Norm.** Every vector has unit length, within floating-point error.

## Comparing

Vectors are normalized, so cosine similarity is a dot product. Compare a new vector with
`centroid` for a quick answer, or with every vector in `embeddings` and take the best score when
the person was recorded through different microphones.

## What is never in the file

Raw audio, text of what was said, the name of the device, the path of the source files.
