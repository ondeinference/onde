// Aggregate one CI run into a markdown table.
//   node report.mjs <artifacts-dir> > summary.md
// <artifacts-dir>/cell-*/        browser cells: cell.json + bench-*.json (run-page output)
// <artifacts-dir>/native-*/      native benches: <device>-<cfg>.jsonl/.err/.exit
// Token match compares the first 20 generated tokens with the native CPU run
// of the same config (greedy decoding, same GGUF).
import fs from "node:fs";
import path from "node:path";
import { execFileSync } from "node:child_process";

const root = process.argv[2];
const here = path.dirname(new URL(import.meta.url).pathname);
const CFGS = ["bench-tiny", "bench-quick", "bench"];
const read = (f) => { try { return fs.readFileSync(f, "utf8"); } catch { return null; } };
const json = (f) => { const s = read(f); try { return s ? JSON.parse(s) : null; } catch { return null; } };
const dirs = (prefix) => fs.existsSync(root) ? fs.readdirSync(root).filter((d) => d.startsWith(prefix)).sort() : [];
const num = (x, d = 1) => (x == null || Number.isNaN(x) ? "–" : Number(x).toFixed(d));
const mean = (a) => a.reduce((x, y) => x + y, 0) / a.length;

const jsonl = (f) => (read(f) || "").split("\n").filter((l) => l.startsWith("{")).map((l) => JSON.parse(l));

// Reference tokens: native CPU, first measured run, per config.
const nativeDirs = dirs("native-");
const ref = {};
for (const d of nativeDirs) for (const cfg of CFGS) {
  const r = jsonl(path.join(root, d, `cpu-${cfg}.jsonl`)).find((x) => !x.warmup);
  if (r && !ref[cfg]) ref[cfg] = r.tokens;
}
const refText = {};
for (const d of nativeDirs) for (const cfg of CFGS) {
  const r = jsonl(path.join(root, d, `cpu-${cfg}.jsonl`)).find((x) => !x.warmup);
  if (r && !refText[cfg]) refText[cfg] = r.text;
}
// Harnesses that cannot expose token ids (wllama) are compared on text: the
// length of the common prefix, in characters, out of the first 60.
function matchText(cfg, text) {
  const r = refText[cfg];
  if (!r || !text) return "no ref";
  const n = Math.min(60, r.length, text.length);
  let i = 0;
  while (i < n && r[i] === text[i]) i++;
  return i === n ? `text ${n}/${n} chars` : `text ${i}/${n} chars`;
}
function match(cfg, tokens) {
  const r = ref[cfg];
  if (!r || !tokens) return "no ref";
  const n = Math.min(20, r.length, tokens.length);
  let i = 0;
  while (i < n && r[i] === tokens[i]) i++;
  return i === n ? `${n}/${n}` : `${i}/${n} (differs at ${i})`;
}

const rows = [];
const detail = [];
// --- native rows
for (const d of nativeDirs) {
  const runner = d.replace(/^native-/, "");
  for (const dev of ["cpu", "metal", "wgpu", "vectoron"]) {
    // A workflow only writes files for the devices it runs.
    if (!fs.existsSync(path.join(root, d, `${dev}-bench-tiny.jsonl`))) continue;
    let top = null;
    for (const cfg of CFGS) {
      const runs = jsonl(path.join(root, d, `${dev}-${cfg}.jsonl`)).filter((x) => !x.warmup);
      if (runs.length) top = { cfg, runs };
    }
    const err = (read(path.join(root, d, `${dev}-bench-tiny.err`)) || "").trim().split("\n").slice(-2).join(" ").slice(0, 160);
    const label = { cpu: "candle CPU (native)", metal: "candle Metal (native)", wgpu: "FerrisMind wgpu (native)", vectoron: "vectoron CPU (native)" }[dev];
    if (!top) { rows.push([label, runner, "–", "none", "–", "–", "–", "–", "–", "–", `failed: ${err || "no output"}`]); continue; }
    const r = top.runs;
    const load = (read(path.join(root, d, `${dev}-${top.cfg}.err`)) || "").match(/load_ms=([\d.]+)/);
    rows.push([label, runner, "–", top.cfg, num(mean(r.map((x) => x.prefill_ms)), 0), num(mean(r.map((x) => x.ttft_ms)), 0),
      num(mean(r.map((x) => x.decode_tps)), 1), load ? num(+load[1], 0) : "–", "–", match(top.cfg, r[0].tokens), "ok"]);
  }
}
// --- browser rows
for (const d of dirs("cell-")) {
  const cell = json(path.join(root, d, "cell.json")) || {};
  const runner = d.includes("ubuntu") ? "ubuntu-latest" : d.includes("macos") ? "macos-26" : "?";
  const label = `${cell.harness || d} (wasm${cell.drain > 1 ? `, drain/${cell.drain}` : ""}${cell.extra ? ", " + cell.extra.replace(/^&/, "") : ""})`;
  let top = null, lastFail = null;
  for (const cfg of CFGS) {
    const r = json(path.join(root, d, `${cfg}.json`));
    if (!r) continue;
    detail.push({ d, cfg, r });
    if (r.error) { lastFail = { cfg, r }; break; }
    top = { cfg, r };
  }
  const t = top?.r;
  const runs = t ? t.result.runs.filter((x) => !x.warmup) : [];
  const first = runs[0];
  const gpu = (lastFail?.r.adapter || t?.adapter);
  const peak = Math.max(0, ...CFGS.map((c) => json(path.join(root, d, `${c}.json`))?.guard?.peak_rss_gb ?? 0));
  const status = lastFail ? `${lastFail.cfg} failed: ${String(lastFail.r.error).split("\n")[0].slice(0, 140)}` : "ok";
  const adapter = gpu ? (gpu.available ? `${gpu.description || gpu.vendor || "adapter"}${gpu.isFallbackAdapter ? " (fallback)" : ""}` : `no WebGPU: ${gpu.reason}`) : "–";
  rows.push([label, runner, cell.browser || "?", top ? top.cfg : "none",
    first ? num(mean(runs.map((x) => x.prefill_ms)), 0) : "–", first ? num(mean(runs.map((x) => x.ttft_ms)), 0) : "–",
    first ? num(mean(runs.map((x) => x.decode_tps)), 1) : "–", t ? num(t.result.load_ms, 0) : "–", num(peak, 2),
    first ? (first.tokens ? match(top.cfg, first.tokens) : matchText(top.cfg, first.text)) : "–", `${status}; ${adapter}`]);
}

const out = [];
out.push("| path | runner | browser | highest rung | prefill ms | TTFT ms | decode tok/s | load ms | peak RSS GB | first 20 tokens vs native CPU | status / adapter |");
out.push("|---|---|---|---|---|---|---|---|---|---|---|");
for (const r of rows) out.push("| " + r.join(" | ") + " |");
out.push("");
out.push("Hosted runners: macos-26 is an Apple Silicon VM with a paravirtualized GPU, ubuntu-latest has no GPU (WebGPU is software SwiftShader). Neither measures a real Mac's GPU; compare paths within a runner, not against local numbers.");
out.push("");
out.push("GPU events (device lost, uncaptured errors) per cell:");
for (const { d, cfg, r } of detail) {
  const ev = r.result?.gpu_events || r.partial?.gpu_events || [];
  if (ev.length) out.push(`- ${d} / ${cfg}: \`${JSON.stringify(ev).slice(0, 500)}\``);
}
out.push("");
for (const { d, cfg, r } of detail) {
  const f = path.join(root, d, `${cfg}.json`);
  let body = "";
  if (!r.error) { try { body = execFileSync("node", [path.join(here, "summarize.mjs"), f], { encoding: "utf8" }); } catch (e) { body = String(e.stdout || e.message); } }
  else body = `ERROR: ${r.error}\n\nconsole tail:\n${(r.console || []).slice(-15).join("\n")}`;
  out.push(`<details><summary>${d} / ${cfg}</summary>\n\n\`\`\`\n${body.trim()}\n\`\`\`\n\nguard: \`${JSON.stringify(r.guard)}\`\n\nadapter: \`${JSON.stringify(r.adapter)}\`\n\n</details>\n`);
}
console.log(out.join("\n"));
