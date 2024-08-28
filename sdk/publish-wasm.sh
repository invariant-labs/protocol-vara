#!/bin/bash

# Usage: ./publish-wasm.sh <version>
# For example: ./publish-wasm.sh 0.1.0

jq '.name = "@invariant-labs/vara-sdk-wasm"' src/wasm/pkg/package.json > temp.json && mv temp.json src/wasm/pkg/package.json

if [ -z "$1" ]; then
    echo "Please provide the version to publish."
    exit 1
fi

jq ".version = \"$1\"" src/wasm/pkg/package.json > temp.json && mv temp.json src/wasm/pkg/package.json

cd src/wasm/pkg
npm publish
