/** Whether this browser exposes a WebGPU adapter. */
export async function hasWebGpu(): Promise<boolean> {
  const gpu = (globalThis.navigator as { gpu?: { requestAdapter(): Promise<unknown> } } | undefined)
    ?.gpu;
  if (!gpu) return false;
  try {
    return (await gpu.requestAdapter()) != null;
  } catch {
    return false;
  }
}

/** Whether wasm simd128 works here. The CPU engine is built with it. */
export function hasWasmSimd(): boolean {
  // (module (func (result v128) i32.const 0 i8x16.splat))
  const probe = new Uint8Array([
    0, 97, 115, 109, 1, 0, 0, 0, 1, 5, 1, 96, 0, 1, 123, 3, 2, 1, 0, 10, 10, 1, 8, 0, 65, 0,
    253, 15, 253, 98, 11,
  ]);
  try {
    return WebAssembly.validate(probe);
  } catch {
    return false;
  }
}
