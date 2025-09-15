#!/bin/bash
GOOS=js GOARCH=wasm go build -o mangle.wasm wasm/main.go
cp "$(go env GOROOT)/lib/wasm/wasm_exec.js" .
