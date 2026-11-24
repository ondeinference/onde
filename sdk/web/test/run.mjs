// Opens the smoke app in headless Chrome and waits for window.__result.
//   node run.mjs "<query>" --out result.json
// Env: BASE (server url), MAX_SECONDS / MAX_RSS_GB (guard), CHROME_ARGS.
// The guard kills the browser past MAX_RSS_GB per process or MAX_SECONDS in
// total; it exists because a runaway WebGPU allocation once took a 24 GiB Mac
// down. CI only: never run this on a developer machine.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { chromium } from "playwright";
import { startGuard } from "./guard.mjs";

const query = process.argv[2] ?? "engine=cpu";
const out = process.argv.includes("--out") ? process.argv[process.argv.indexOf("--out") + 1] : null;
const base = process.env.BASE ?? "http://127.0.0.1:8123";
const maxSeconds = Number(process.env.MAX_SECONDS ?? 600);
const timeout = Math.max(10, maxSeconds - 10) * 1000;

let guardKill = null;
const guard = startGuard({ maxSeconds, onKill: (why) => (guardKill = why) });
const consoleLog = [];
const note = (s) => {
  if (consoleLog.length < 500) consoleLog.push(s);
  console.error(s);
};

let ctx;
let res = {};
try {
  const profile = fs.mkdtempSync(path.join(os.tmpdir(), "onde-web-smoke-"));
  const extra = (process.env.CHROME_ARGS ?? "").split(/\s+/).filter(Boolean);
  ctx = await chromium.launchPersistentContext(profile, {
    channel: "chrome",
    headless: true,
    args: ["--enable-unsafe-webgpu", "--enable-features=Vulkan,WebGPU", ...extra],
  });
  const page = await ctx.newPage();
  page.on("console", (m) => note("[console] " + m.text()));
  page.on("pageerror", (e) => note("[pageerror] " + e.message));
  try {
    await page.goto(`${base}/?${query}`);
    res.adapter = await page
      .evaluate(async () => {
        if (!("gpu" in navigator)) return { available: false, reason: "navigator.gpu missing" };
        const a = await navigator.gpu.requestAdapter();
        if (!a) return { available: false, reason: "requestAdapter() returned null" };
        const i = a.info ?? {};
        return { available: true, vendor: i.vendor, architecture: i.architecture, description: i.description };
      })
      .catch((e) => ({ available: false, reason: "probe threw: " + e.message }));
    await page.evaluate(() => {
      window.__start();
    });
    await page.waitForFunction(() => window.__result || window.__error, null, { timeout, polling: 1000 });
    res = {
      ...res,
      ...(await page.evaluate(() => ({ result: window.__result, error: window.__error, partial: window.__partial }))),
    };
  } catch (e) {
    res.error = guardKill ? `killed by guard: ${guardKill}` : `driver: ${e.message}`;
  }
} catch (e) {
  res.error = guardKill ? `killed by guard: ${guardKill}` : `launch: ${e.message}`;
}
res.guard = guard.stop();
res.console = consoleLog;
if (out) fs.writeFileSync(out, JSON.stringify(res, null, 2));
console.log(JSON.stringify({ error: res.error, result: res.result, guard: res.guard, adapter: res.adapter }, null, 2));
await ctx?.close().catch(() => {});
process.exit(res.error ? 1 : 0);
