#!/usr/bin/env bash
# The quality gate: everything here has to pass before a push.
set -euo pipefail

cd "$(dirname "$0")/.."

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

for script in ui/*.js; do
  node --check "$script"
done
python3 -c "import json, sys; [json.load(open(f)) for f in sys.argv[1:]]" \
  schema/qvoice.v1.schema.json app/tauri.conf.json app/capabilities/default.json

echo "All checks passed."
