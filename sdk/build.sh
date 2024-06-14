#!/bin/bash

npm install
npm run lint
npm run wasm:build
npm run contract:build
npm run erc-20:copy
npm run erc-20:generate
npm run build