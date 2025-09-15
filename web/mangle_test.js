import {
  assert,
  assertEquals,
} from "https://deno.land/std@0.128.0/testing/asserts.ts";
import "./wasm_exec.js";

async function runMangleInstance(wasmPath) {
  const go = new Go();
  const wasmBytes = await Deno.readFile(wasmPath);
  const wasmModule = new WebAssembly.Module(wasmBytes);
  const instance = new WebAssembly.Instance(wasmModule, go.importObject);
  go.run(instance);
  return instance;
}

Deno.test("mangle wasm", async () => {
  await runMangleInstance("web/mangle.wasm");

  let err = mangleDefine('foo(1, 2).');
  assertEquals(err, null);

  err = mangleDefine('bar("baz").');
  assertEquals(err, null);

  const result = mangleQuery("foo(X, Y)");
  assertEquals(result.trim(), "foo(1,2)");

  const result2 = mangleQuery("bar(X)");
  assertEquals(result2.trim(), 'bar("baz")');

  const errResult = mangleQuery("foo(");
  assert(errResult.startsWith("Error:"));
});
