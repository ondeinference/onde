// Kill switch for browser runs. Polls RSS every 500 ms for every process that
// descends from the driver (so the browser, its GPU and renderer helpers, on
// any platform) and kills them all if any single process passes MAX_RSS_GB
// (default 4) or the run passes MAX_SECONDS (default 120). Exists because on
// 2026-11-17 a runaway WebGPU allocation (~49 GB in Chrome's GPU process)
// panicked a 24 GiB Mac. Watching only descendants of this process means an
// everyday browser is never touched.
//
// Library use:  const g = startGuard({ onKill }); ...; const report = g.stop();
import { execFileSync } from "node:child_process";

function descendants(rootPid) {
  let out = "";
  try { out = execFileSync("ps", ["-axo", "pid=,ppid=,rss=,command="], { encoding: "utf8", maxBuffer: 64 << 20 }); }
  catch { return []; }
  const all = out.split("\n").flatMap((l) => {
    const m = l.match(/^\s*(\d+)\s+(\d+)\s+(\d+)\s+(.*)$/);
    return m ? [{ pid: +m[1], ppid: +m[2], rssKB: +m[3], cmd: m[4] }] : [];
  });
  const watched = new Map();
  for (const p of all) if (p.ppid === rootPid) watched.set(p.pid, p);
  for (let grew = true; grew; ) {
    grew = false;
    for (const p of all) if (!watched.has(p.pid) && watched.has(p.ppid)) { watched.set(p.pid, p); grew = true; }
  }
  return [...watched.values()];
}

export function startGuard({ maxGB = Number(process.env.MAX_RSS_GB || 4), maxSeconds = Number(process.env.MAX_SECONDS || 120), onKill } = {}) {
  const t0 = Date.now();
  let peakKB = 0, peakCmd = "", killed = null;
  const tick = () => {
    const procs = descendants(process.pid);
    for (const p of procs) {
      if (p.rssKB > peakKB) { peakKB = p.rssKB; peakCmd = (p.cmd.match(/--type=([\w-]+)/) || [, "browser"])[1]; }
    }
    const over = procs.find((p) => p.rssKB > maxGB * 1024 * 1024);
    const secs = (Date.now() - t0) / 1000;
    if (!killed && (over || secs > maxSeconds)) {
      killed = over ? `process ${over.pid} RSS ${(over.rssKB / 1048576).toFixed(2)} GB > ${maxGB} GB` : `run exceeded ${maxSeconds} s`;
      console.error(`[guard] KILL: ${killed}`);
      for (const p of procs) { try { process.kill(p.pid, "SIGKILL"); } catch {} }
      onKill?.(killed);
    }
  };
  const timer = setInterval(tick, 500);
  tick();
  return {
    stop() {
      clearInterval(timer);
      const r = { peak_rss_gb: +(peakKB / 1048576).toFixed(3), peak_process: peakCmd, killed, seconds: +((Date.now() - t0) / 1000).toFixed(1), max_rss_gb: maxGB, max_seconds: maxSeconds };
      console.error("[guard] " + JSON.stringify(r));
      return r;
    },
  };
}
