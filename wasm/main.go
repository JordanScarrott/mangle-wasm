package main

import (
	"bytes"
	"strings"
	"syscall/js"

	"github.com/google/mangle/interpreter"
)

var i *interpreter.Interpreter

func define(this js.Value, args []js.Value) interface{} {
	if len(args) != 1 {
		return "Invalid number of arguments"
	}
	err := i.Define(args[0].String())
	if err != nil {
		return "Error: " + err.Error()
	}
	return nil
}

func query(this js.Value, args []js.Value) interface{} {
	if len(args) != 1 {
		return "Invalid number of arguments"
	}
	query, err := i.ParseQuery(args[0].String())
	if err != nil {
		return "Error: " + err.Error()
	}
	res, err := i.Query(query)
	if err != nil {
		return "Error: " + err.Error()
	}
	var results []string
	for _, fact := range res {
		results = append(results, fact.String())
	}
	return strings.Join(results, "\n")
}

func main() {
	var writer bytes.Buffer
	i = interpreter.New(&writer, "", nil)
	c := make(chan struct{}, 0)
	js.Global().Set("mangleDefine", js.FuncOf(define))
	js.Global().Set("mangleQuery", js.FuncOf(query))
	<-c
}
