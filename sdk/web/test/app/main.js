// Smoke page, driven by ../run.mjs. Query: engine=cpu|webgpu, ngl=<gpu layers>.
// Downloads the model through the package, loads it greedy, asks one question
// twice (with a reset between) and publishes the result on window.__result.
import { createOnde } from "@ondeinference/web";
import { webgpu } from "@ondeinference/web/webgpu";

const QUESTION = "Name one planet in the solar system.";
const log = (s) => {
  document.getElementById("log").textContent += s + "\n";
  console.log(s);
};
const q = new URLSearchParams(location.search);
const engine = q.get("engine") ?? "cpu";
const out = { engine, question: QUESTION };
window.__partial = out;

window.__start = async () => {
  try {
    const onde = await createOnde({
      engine,
      webgpu: webgpu(q.has("ngl") ? { gpuLayers: Number(q.get("ngl")) } : {}),
    });
    out.model = await onde.model();
    out.cachedBefore = await onde.isCached();

    let t0 = performance.now();
    let lastLog = 0;
    await onde.download((received, total) => {
      if (performance.now() - lastLog > 5000) {
        lastLog = performance.now();
        log(`download ${(received / 1e6).toFixed(0)} / ${total ? (total / 1e6).toFixed(0) : "?"} MB`);
      }
    });
    out.download_ms = performance.now() - t0;
    out.cachedAfter = await onde.isCached();

    out.load_s = await onde.load({ sampling: { temperature: 0, maxTokens: 20 } });
    out.engineAfterLoad = onde.engine;
    log(`loaded on ${onde.engine} in ${out.load_s.toFixed(1)} s`);

    t0 = performance.now();
    const pieces = [];
    for await (const delta of onde.stream(QUESTION)) pieces.push(delta);
    out.first = { text: pieces.join(""), pieces: pieces.length, ms: performance.now() - t0 };
    log(`first: ${JSON.stringify(out.first.text)}`);

    out.dropped = await onde.reset();
    t0 = performance.now();
    const second = await onde.send(QUESTION);
    out.second = { text: second, ms: performance.now() - t0 };
    out.repeatable = second === out.first.text;
    out.info = await onde.info();
    await onde.dispose();
    window.__result = out;
  } catch (e) {
    window.__error = String((e && e.stack) || e);
  }
};
