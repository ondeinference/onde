// node summarize.mjs results/<file>.json  -> markdown rows (mean ± sd over measured runs)
import fs from "node:fs";
const r = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
if (r.error) { console.log("ERROR:", r.error); process.exit(1); }
const res = r.result;
const stat = (a) => { const m = a.reduce((x, y) => x + y, 0) / a.length;
  const sd = Math.sqrt(a.reduce((x, y) => x + (y - m) ** 2, 0) / Math.max(1, a.length - 1)); return `${m.toFixed(1)} ± ${sd.toFixed(1)}`; };
console.log("UA:", r.userAgent);
for (const k of ["sha256","read_ms","wasm_init_ms","load_ms","wasm_bytes","wasm_memory_bytes","wasm_memory_bytes_after","seeded"]) if (k in res) console.log(k, res[k]);
const groups = {};
for (const x of res.runs.filter((x) => !x.warmup)) (groups[x.yielded] ||= []).push(x);
console.log("| yield | n | prompt | prefill ms | TTFT ms | decode tok/s | turn tok/s |\n|---|---|---|---|---|---|---|");
for (const [y, g] of Object.entries(groups))
  console.log(`| ${y} | ${g.length} | ${g[0].prompt_tokens} | ${stat(g.map((x) => x.prefill_ms))} | ${stat(g.map((x) => x.ttft_ms))} | ${stat(g.map((x) => x.decode_tps))} | ${stat(g.map((x) => x.turn_tps))} |`);
const first = res.runs.find((x) => !x.warmup);
console.log("first 20 tokens:", JSON.stringify(first.tokens.slice(0, 20)));
