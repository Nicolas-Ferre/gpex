#!/bin/bash
set -xeu

echo "value=$(grep -i rust-version Cargo.toml | cut -d '"' -f2 | tr -d '\n')" >>"$GITHUB_OUTPUT"
