// Module worker that owns the wasm module and the one CpuEngine. Loading the
// model and every forward pass block this thread, never the page's.

import init, {
  CpuEngine,
  delete_cached,
  download,
  is_cached,
  model_info,
} from "../../wasm/onde_web.js";
import type { Reply, Request } from "./protocol.js";

const scope = self as unknown as {
  postMessage(message: Reply): void;
  onmessage: ((event: MessageEvent<Request>) => void) | null;
};

const ready = init().then(() => new CpuEngine());

async function handle(engine: CpuEngine, request: Request): Promise<unknown> {
  const { id } = request;
  switch (request.type) {
    case "model":
      return model_info();
    case "isCached":
      return is_cached();
    case "download":
      return download((received: number, total: number | null) =>
        scope.postMessage({ id, type: "progress", received, total }),
      );
    case "deleteCached":
      return delete_cached();
    case "load":
      return engine.load(request.options.systemPrompt, request.options.sampling);
    case "send":
      return engine.send(request.message, (text: string) =>
        scope.postMessage({ id, type: "delta", text }),
      );
    case "reset":
      return engine.reset();
    case "info":
      return engine.info();
  }
}

scope.onmessage = async ({ data }) => {
  try {
    const value = await handle(await ready, data);
    scope.postMessage({ id: data.id, type: "result", value });
  } catch (error) {
    const message = String((error as Error)?.message ?? error);
    scope.postMessage({ id: data.id, type: "error", message });
  }
};
