#!/usr/bin/env bash
# Downloads the pinned models into models/ and checks their checksums.
set -euo pipefail

cd "$(dirname "$0")/.."
mkdir -p models

base=https://github.com/k2-fsa/sherpa-onnx/releases/download

fetch() {
  local file=$1 url=$2 sha=$3
  if [ ! -f "models/$file" ]; then
    curl -fL --retry 3 -o "models/$file" "$url"
  fi
  if command -v sha256sum >/dev/null; then
    echo "$sha  models/$file" | sha256sum -c -
  else
    echo "$sha  models/$file" | shasum -a 256 -c -
  fi
}

fetch 3dspeaker_speech_campplus_sv_zh_en_16k-common_advanced.onnx \
  "$base/speaker-recongition-models/3dspeaker_speech_campplus_sv_zh_en_16k-common_advanced.onnx" \
  aa3cfc16963a10586a9393f5035d6d6b57e98d358b347f80c2a30bf4f00ceba2
fetch silero_vad.onnx \
  "$base/asr-models/silero_vad.onnx" \
  9e2449e1087496d8d4caba907f23e0bd3f78d91fa552479bb9c23ac09cbb1fd6
