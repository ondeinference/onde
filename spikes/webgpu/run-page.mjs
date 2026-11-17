// Generic driver: open a harness page in a browser and wait for window.__result.
//   node run-page.mjs <url-path> <chrome|webkit> --out file.json
// Env: BASE (server url), PROFILE_DIR (Chrome profile, default a temp dir),
//      CHROME_ARGS (extra flags, space separated), MAX_SECONDS / MAX_RSS_GB (guard).
// The guard always runs: it kills the browser past MAX_RSS_GB per process or
// MAX_SECONDS overall, and its peak RSS lands in the result JSON.
// "webkit" = Playwright's WebKit build, not Safari.app.
import { chromium, webkit } from "playwright";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { startGuard } from "./guard.mjs";

const [pagePath, browserName = "chrome"] = process.argv.slice(2);
const out = process.argv.includes("--out") ? process.argv[process.argv.indexOf("--out") + 1] : null;
const base = process.env.BASE || "http://127.0.0.1:8123";
const maxSeconds = Number(process.env.MAX_SECONDS || 120);
// Give up a little before the guard's own deadline so a slow run reports a driver timeout, not a kill.
const timeout = Number(process.env.TIMEOUT_MS || Math.max(10, maxSeconds - 10) * 1000);

let guardKill = null;
const guard = startGuard({ maxSeconds, onKill: (why) => { guardKill = why; } });
const consoleLog = [];
const note = (s) => { if (consoleLog.length < 500) consoleLog.push(s); console.error(s); };

let ctx, res = {};
try {
  if (browserName === "chrome") {
    const profile = process.env.PROFILE_DIR || fs.mkdtempSync(path.join(os.tmpdir(), "profile-chrome-"));
    const extra = (process.env.CHROME_ARGS || "").split(/\s+/).filter(Boolean);
    ctx = await chromium.launchPersistentContext(profile, {
      channel: "chrome",
      headless: true,
      args: ["--enable-unsafe-webgpu", "--enable-features=Vulkan,WebGPU", "--enable-webgpu-developer-features", ...extra],
    });
  } else {
    const b = await webkit.launch({ headless: true });
    ctx = await b.newContext();
  }
  const page = await ctx.newPage();
  page.on("console", (m) => note("[console] " + m.text()));
  page.on("pageerror", (e) => note("[pageerror] " + e.message));
  try {
    await page.goto(base + pagePath);
    // Adapter probe from the page itself, recorded for every cell (also the CPU ones).
    res.adapter = await page.evaluate(async () => {
      if (!("gpu" in navigator)) return { available: false, reason: "navigator.gpu missing" };
      const a = await navigator.gpu.requestAdapter();
      if (!a) return { available: false, reason: "requestAdapter() returned null" };
      const i = a.info || {};
      return { available: true, vendor: i.vendor, architecture: i.architecture, device: i.device, description: i.description,
        isFallbackAdapter: a.isFallbackAdapter, maxBufferSize: a.limits.maxBufferSize };
    }).catch((e) => ({ available: false, reason: "probe threw: " + e.message }));
    await page.evaluate(() => { window.__start && window.__start(); }); // not awaited: wllama's start is long-running
    await page.waitForFunction(() => window.__result || window.__error, null, { timeout, polling: 1000 });
    res = { ...res, ...(await page.evaluate(() => ({ result: window.__result, error: window.__error, partial: window.__partial }))) };
    res.userAgent = await page.evaluate(() => navigator.userAgent);
  } catch (e) {
    res.error = guardKill ? `killed by guard: ${guardKill}` : `driver: ${e.message}`;
  }
} catch (e) {
  res.error = guardKill ? `killed by guard: ${guardKill}` : `launch: ${e.message}`;
}
res.guard = guard.stop();
res.console = consoleLog;
if (out) fs.writeFileSync(out, JSON.stringify(res, null, 2));
console.log(JSON.stringify({ error: res.error, guard: res.guard, adapter: res.adapter }).slice(0, 4000));
await ctx?.close().catch(() => {});
process.exit(res.error ? 1 : 0);
