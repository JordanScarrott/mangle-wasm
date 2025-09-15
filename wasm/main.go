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
