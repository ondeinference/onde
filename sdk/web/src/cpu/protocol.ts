// Messages between CpuEngine (page) and the CPU worker. Every request carries
// an id; the worker answers with a "result" or "error" for that id, and sends
// "progress" or "delta" events with the same id while the request runs.

import type { LoadOptions } from "../types.js";

export type Request =
  | { id: number; type: "model" }
  | { id: number; type: "isCached" }
  | { id: number; type: "download" }
  | { id: number; type: "deleteCached" }
  | { id: number; type: "load"; options: LoadOptions }
  | { id: number; type: "send"; message: string }
  | { id: number; type: "reset" }
  | { id: number; type: "info" };

export type Reply =
  | { id: number; type: "result"; value: unknown }
  | { id: number; type: "error"; message: string }
  | { id: number; type: "progress"; received: number; total: number | null }
  | { id: number; type: "delta"; text: string };
