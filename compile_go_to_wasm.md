# Compiling the Go implementation of Mangle to WebAssembly

## Quick Start: How to Compile to WebAssembly

This guide assumes you have a working Go environment set up.

1.  **Clone the repository:**
    (This step is not necessary if you are already in the repository)
    ```bash
    git clone https://github.com/google/mangle.git
    cd mangle
    ```

2.  **Run the build script:**
    ```bash
    ./build_wasm.sh
    ```

3.  **Verify the output:**
    After running the script, you will find the following files in the root directory of the repository:
    *   `mangle.wasm`: The compiled WebAssembly module.
    *   `wasm_exec.js`: The Go WebAssembly JavaScript support file.

---

**Note:** The steps outlined in this document have already been implemented. This document serves as a record of the work that was done and as a guide for future developers who may want to understand or replicate the process.

## Instructional Prompt for Jules 2.0

Your task is to compile the Go implementation of Google Mangle to WebAssembly. This will involve creating a new entry point for the WASM build, exposing the interpreter's functionality to JavaScript, and creating a build script to automate the compilation process. You will also need to create a simple HTML file to test the resulting WASM module.

## Findings and Step-by-Step Guide

This document outlines the process of compiling the Go implementation of Google Mangle to WebAssembly.

### 1. Create a new entry point for the WASM build

The existing `main` package in `interpreter/mg/mg.go` is designed for a command-line interface and is not suitable for a WebAssembly build. We need to create a new `main` package that will serve as the entry point for the WASM module.

1.  Create a new directory named `wasm`.
2.  Create a new file named `wasm/main.go` with the following content:

```go
package main

import (
	"bytes"
	"strings"
	"syscall/js"

	"github.com/google/mangle/interpreter"
)

func runMangle(this js.Value, args []js.Value) interface{} {
	if len(args) != 1 {
		return "Invalid number of arguments"
	}
	queryStr := args[0].String()
	var writer bytes.Buffer
	i := interpreter.New(&writer, "", nil)
	query, err := i.ParseQuery(queryStr)
	if err != nil {
		return "Error parsing query: " + err.Error()
	}
	res, err := i.Query(query)
	if err != nil {
		return "Error evaluating query: " + err.Error()
	}
	var results []string
	for _, fact := range res {
		results = append(results, fact.String())
	}
	return strings.Join(results, "\n")
}

func main() {
	c := make(chan struct{}, 0)
	js.Global().Set("runMangle", js.FuncOf(runMangle))
	<-c
}
```

This code creates a `runMangle` function that takes a Mangle query as a string, executes it using the interpreter, and returns the result as a string. The `main` function registers this function as a global JavaScript function named `runMangle`.

### 2. Refactor the interpreter package

The `interpreter` package has a dependency on the `github.com/chzyer/readline` package, which is not compatible with WebAssembly. We need to use build constraints to create a version of the interpreter that does not have this dependency.

1.  Rename `interpreter/interpreter.go` to `interpreter/interpreter_notjs.go`.
2.  Add the following build constraint to the top of `interpreter/interpreter_notjs.go`:

```go
//go:build !js
// +build !js
```

3.  Create a new file named `interpreter/interpreter.go` and add a build constraint to it to ensure it is only used for wasm builds:

```go
//go:build js && wasm
// +build js,wasm
```

4.  Copy the content of `interpreter/interpreter_notjs.go` to `interpreter/interpreter.go`, but remove the `Loop` function and the import of `github.com/chzyer/readline`.

### 3. Create a build script

Create a build script named `build_wasm.sh` to automate the compilation process.

```bash
#!/bin/bash
GOOS=js GOARCH=wasm go build -o mangle.wasm wasm/main.go
cp "$(go env GOROOT)/lib/wasm/wasm_exec.js" .
```

Make the script executable:

```bash
chmod +x build_wasm.sh
```

### 4. Create an HTML file for testing

Create an `index.html` file to test the WebAssembly module.

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <script src="wasm_exec.js"></script>
    <script>
        const go = new Go();
        WebAssembly.instantiateStreaming(fetch("mangle.wasm"), go.importObject).then((result) => {
            go.run(result.instance);
        });

        function run() {
            const query = document.getElementById("query").value;
            const result = runMangle(query);
            document.getElementById("output").innerText = result;
        }
    </script>
</head>
<body>
    <h1>Mangle WASM</h1>
    <textarea id="query" rows="10" cols="80"></textarea>
    <br>
    <button onClick="run()">Run</button>
    <pre id="output"></pre>
</body>
</html>
```

### 5. Build and run

1.  Run the build script:

```bash
./build_wasm.sh
```

2.  Serve the `index.html`, `wasm_exec.js`, and `mangle.wasm` files from a web server. For example, you can use Python's built-in HTTP server:

```bash
python3 -m http.server
```

3.  Open your browser and navigate to `http://localhost:8000`. You should see a text area where you can enter Mangle queries and a button to execute them.
