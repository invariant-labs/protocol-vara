#!/bin/bash
set -e 

npm install
npm run lint
npm run wasm:build
./package-wasm.sh
npm run contract:build
npm run erc-20:generate
npm run invariant:generate
npm run fix-generate
npm run build