#!/bin/bash

if [ -z "$1" ]
  then
    TYPE=debug
fi

cargo build --manifest-path=examples/basic/Cargo.toml --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/debug/nukly.wasm  --out-dir generated --no-modules
wasm-bindgen target/wasm32-unknown-unknown/debug/nukly-example-basic.wasm  --out-dir generated --no-modules
cp examples/basic/index.html generated/