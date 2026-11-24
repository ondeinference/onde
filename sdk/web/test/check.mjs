// Asserts on result JSON files written by run.mjs.
//   node check.mjs cpu result-cpu.json
//   node check.mjs compare result-cpu.json result-webgpu.json   (report only)
import fs from "node:fs";

const [mode, ...files] = process.argv.slice(2);
const read = (f) => JSON.parse(fs.readFileSync(f, "utf8"));
const fail = (why) => {
  console.error(`::error::${why}`);
  process.exit(1);
};

if (mode === "cpu") {
  const { error, result: r } = read(files[0]);
  if (error) fail(`smoke run failed: ${error.split("\n")[0]}`);
  if (!r.cachedAfter) fail("model is not cached after download()");
  if (r.engineAfterLoad !== "cpu") fail(`expected the cpu engine, got ${r.engineAfterLoad}`);
  if (!r.first.text.trim()) fail("empty reply");
  if (r.first.pieces < 2) fail(`reply did not stream (${r.first.pieces} pieces)`);
  if (!r.repeatable) fail(`greedy replies differ: ${JSON.stringify(r.first.text)} vs ${JSON.stringify(r.second.text)}`);
  if (r.dropped !== 2) fail(`reset dropped ${r.dropped} messages, expected 2`);
  console.log(`ok: ${JSON.stringify(r.first.text)} in ${(r.first.ms / 1000).toFixed(1)} s`);
} else if (mode === "compare") {
  const cpu = read(files[0]).result;
  const gpu = read(files[1]);
  const lines = [`| | cpu | webgpu |`, `|---|---|---|`];
  lines.push(`| engine | ${cpu?.engineAfterLoad} | ${gpu.result?.engineAfterLoad ?? "–"} |`);
  lines.push(`| reply | ${JSON.stringify(cpu?.first.text)} | ${JSON.stringify(gpu.result?.first.text ?? gpu.error?.split("\n")[0])} |`);
  lines.push(`| ms | ${cpu?.first.ms.toFixed(0)} | ${gpu.result?.first.ms.toFixed(0) ?? "–"} |`);
  lines.push(`| same text | | ${gpu.result ? gpu.result.first.text === cpu?.first.text : "–"} |`);
  const table = lines.join("\n");
  console.log(table);
  if (process.env.GITHUB_STEP_SUMMARY) fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, `### WebGPU vs CPU\n\n${table}\n`);
} else {
  fail(`unknown mode ${mode}`);
}
