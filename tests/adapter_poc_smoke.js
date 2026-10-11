const assert = require("node:assert/strict");
const fs = require("node:fs");
const vm = require("node:vm");

const generatedPath = process.argv[2];
if (!generatedPath) throw new Error("usage: node tests/adapter_poc_smoke.js <generated-js>");
const source = fs.readFileSync(generatedPath, "utf8");
const context = {};
vm.runInNewContext(source, context, { filename: generatedPath });
assert.equal(typeof context.normalizeUplink, "function");

// The Things Stack calls normalizeUplink with the decoder result under input.data.
assert.deepEqual(JSON.parse(JSON.stringify(context.normalizeUplink({ data: { temp_c_dC: 235 } }))),
  { data: { ambient_temperature_mC: 23500 } });
assert.deepEqual(JSON.parse(JSON.stringify(context.normalizeUplink({ data: { temp_f_dF: 743 } }))),
  { data: { ambient_temperature_mC: 23500 } });
assert.deepEqual(JSON.parse(JSON.stringify(context.normalizeUplink({ data: { temp_f_dF: 320 } }))),
  { data: { ambient_temperature_mC: 0 } });

// A caller-supplied canonical field must not override the value derived from a source field.
assert.deepEqual(JSON.parse(JSON.stringify(context.normalizeUplink({
  data: { temp_c_dC: 235, ambient_temperature_mC: 999 }
}))), { data: { ambient_temperature_mC: 23500 } });

for (const payload of [
  {},
  { temp_c_dC: 235, temp_f_dF: 743 },
  { temp_c_dC: "235" },
  { temp_c_dC: 1000000 },
  { temp_c_dC: Number.MAX_SAFE_INTEGER },
  { ambient_temperature_mC: 999 },
]) {
  const result = context.normalizeUplink({ data: payload });
  assert.ok(Array.isArray(result.errors), JSON.stringify({ payload, result }));
  assert.deepEqual(JSON.parse(JSON.stringify(result.data)), {});
}

console.log("adapter PoC runtime smoke test passed (valid, canonical injection, missing, ambiguous, non-integer, range)");
