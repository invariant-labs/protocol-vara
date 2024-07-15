#!/bin/bash
set -e 

npm install
# npm run lint
npm run wasm:build
./package-wasm.sh
# second install for wasm to be added
npm install
npm run contract:build
npm run erc-20:generate
npm run invariant:generate
npm run fix-generate
npm run build