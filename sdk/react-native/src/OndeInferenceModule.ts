// Copyright 2026 Splitfire AB (Onde Inference). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

import { requireNativeModule } from "expo-modules-core";

/**
 * Raw native module interface. All JSON serialization/deserialization
 * happens in the TypeScript wrapper layer (index.ts), not here.
 *
 * Methods that return complex types return JSON strings from the native side.
 * The TypeScript API layer parses these into typed objects.
 */
export interface OndeInferenceNativeModule {
  // Sync
  isLoaded(): boolean;
  setSystemPrompt(prompt: string): void;
  clearSystemPrompt(): void;
  setSampling(samplingJson: string): void;
  clearHistory(): number;
  pushHistory(messageJson: string): void;

  // Async (returns JSON strings)
  loadDefaultModel(
    systemPrompt: string | null,
    samplingJson: string | null,
  ): Promise<string>;
  loadAssignedModel(
    appId: string,
    appSecret: string,
    systemPrompt: string | null,
    samplingJson: string | null,
  ): Promise<string>;
  loadModel(
    configJson: string,
    systemPrompt: string | null,
    samplingJson: string | null,
  ): Promise<string>;
  unloadModel(): Promise<string>;
  info(): Promise<string>;
  history(): Promise<string>;
  sendMessage(message: string): Promise<string>;
  generate(messagesJson: string, samplingJson: string | null): Promise<string>;

  // Config functions (sync, return JSON strings)
  defaultModelConfig(): string;
  qwen251_5bConfig(): string;
  qwen253bConfig(): string;
  qwen3_0_6bConfig(): string;
  qwen3_1_7bConfig(): string;
  qwen3_4bConfig(): string;
  qwen3_8bConfig(): string;
  qwen3_14bConfig(): string;
  qwen3_32bConfig(): string;
  qwen3_4bInstruct2507Config(): string;
  qwen3_4bThinking2507Config(): string;
  qwen3_30bA3bInstruct2507Config(): string;
  defaultSamplingConfig(): string;
  deterministicSamplingConfig(): string;
  mobileSamplingConfig(): string;
}

export default requireNativeModule<OndeInferenceNativeModule>("OndeInference");
