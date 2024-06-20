#!/bin/bash

set -e

cd ./src/wasm/pkg
jq '. + {"type": "module", "main": "invariant_vara_wasm.js"}' package.json > temp
mv temp package.json

echo 'import * as js from "./invariant_vara_wasm.js"' >> invariant_vara_wasm.js
echo 'export default js' >> invariant_vara_wasm.js