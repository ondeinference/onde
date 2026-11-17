// Static dev server for the spike pages.
//   MODEL_DIR=<dir> node serve.mjs <port> [--isolated]
// Serves spikes/webgpu/** and /model/* (from MODEL_DIR, which holds model.gguf
// and tokenizer.json; default ./model next to this file).
import http from "node:http";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));
const modelDir = path.resolve(process.env.MODEL_DIR || path.join(root, "model"));
const port = Number(process.argv[2] || 8123);
const isolated = process.argv.includes("--isolated");
const types = { ".html": "text/html", ".js": "text/javascript", ".mjs": "text/javascript",
  ".wasm": "application/wasm", ".json": "application/json", ".gguf": "application/octet-stream" };

http.createServer((req, res) => {
  const url = new URL(req.url, "http://x");
  let file = url.pathname.startsWith("/model/")
    ? path.join(modelDir, path.basename(url.pathname))
    : path.join(root, path.normalize(url.pathname).replace(/^(\.\.[/\\])+/, ""));
  if (fs.existsSync(file) && fs.statSync(file).isDirectory()) file = path.join(file, "index.html");
  if (!fs.existsSync(file)) { res.writeHead(404); return res.end("not found"); }
  const h = { "content-type": types[path.extname(file)] || "application/octet-stream",
    "content-length": fs.statSync(file).size, "cache-control": "no-store" };
  if (isolated) { h["cross-origin-opener-policy"] = "same-origin"; h["cross-origin-embedder-policy"] = "require-corp"; }
  res.writeHead(200, h);
  fs.createReadStream(file).pipe(res);
}).listen(port, "127.0.0.1", () => console.log(`serving on http://127.0.0.1:${port}`));
