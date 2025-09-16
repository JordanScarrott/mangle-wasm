package main

import (
	"bytes"
	"encoding/json"
	"syscall/js"

	"github.com/google/mangle/ast"
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
	parsedQuery, err := i.ParseQuery(args[0].String())
	if err != nil {
		return "Error: " + err.Error()
	}

	solutions, err := i.Query(parsedQuery)
	if err != nil {
		return "Error: " + err.Error()
	}

	var results []map[string]string

	for _, resultAtom := range solutions {
		bindings := make(map[string]string)

		for i, queryTerm := range parsedQuery.Args {
			switch term := queryTerm.(type) {
			case ast.Variable:
				resultTerm := resultAtom.Args[i]
				// FINAL CORRECTION: Call the .String() method on the variable struct.
				bindings[term.String()] = resultTerm.String()
			}
		}
		if len(bindings) > 0 {
			results = append(results, bindings)
		}
	}

	if len(results) == 0 && len(solutions) > 0 {
		return "[{}]"
	}

	jsonResult, err := json.Marshal(results)
	if err != nil {
		return "Error: " + err.Error()
	}

	return string(jsonResult)
}

func main() {
	var writer bytes.Buffer
	i = interpreter.New(&writer, "", nil)
	c := make(chan struct{}, 0)
	js.Global().Set("mangleDefine", js.FuncOf(define))
	js.Global().Set("mangleQuery", js.FuncOf(query))
	<-c
}