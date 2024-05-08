#!/bin/bash

npm install &&
npm run lint &&
npm run wasm:build &&
npm run build