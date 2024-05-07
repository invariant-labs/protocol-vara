#!/bin/bash

npm i &
npm run lint &
npm run wasm:build &
npm run build &