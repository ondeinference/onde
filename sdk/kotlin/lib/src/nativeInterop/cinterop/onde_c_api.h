// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Thin C API for Kotlin/Native cinterop.
// This header is consumed by the Kotlin/Native cinterop tool to generate
// Kotlin bindings that call directly into the Rust static library.

#ifndef ONDE_C_API_H
#define ONDE_C_API_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// ── Opaque engine handle ─────────────────────────────────────────────────────
typedef void* OndeEngineHandle;

// ── Engine lifecycle ─────────────────────────────────────────────────────────
OndeEngineHandle onde_engine_new(void);
void onde_engine_destroy(OndeEngineHandle engine);

// ── Environment setup ────────────────────────────────────────────────────────
void onde_engine_setup(const char* data_dir);

// ── Model lifecycle ──────────────────────────────────────────────────────────
// Returns load duration in seconds, or -1.0 on error.
double onde_engine_load_default_model(OndeEngineHandle engine, const char* system_prompt);
char* onde_engine_unload(OndeEngineHandle engine);
bool onde_engine_is_loaded(OndeEngineHandle engine);

// ── Inference ────────────────────────────────────────────────────────────────
// Returns reply text (caller frees with onde_string_free), NULL on error.
char* onde_engine_chat(OndeEngineHandle engine, const char* message, double* out_duration_secs);

// Streaming callback type. Return true to continue, false to cancel.
typedef bool (*OndeStreamCallback)(const char* delta, bool done, void* user_data);
// Returns 0 on success, -1 on error.
int32_t onde_engine_stream(OndeEngineHandle engine, const char* message,
                           OndeStreamCallback callback, void* user_data);

// ── Engine state ─────────────────────────────────────────────────────────────
char* onde_engine_status(OndeEngineHandle engine);
char* onde_engine_model_name(OndeEngineHandle engine);
char* onde_engine_approx_memory(OndeEngineHandle engine);
uint64_t onde_engine_history_length(OndeEngineHandle engine);

// ── History ──────────────────────────────────────────────────────────────────
uint64_t onde_engine_clear_history(OndeEngineHandle engine);
void onde_engine_set_system_prompt(OndeEngineHandle engine, const char* prompt);

// ── Cleanup ──────────────────────────────────────────────────────────────────
void onde_string_free(char* s);

// ── Error reporting ──────────────────────────────────────────────────────────
const char* onde_last_error(void);

#ifdef __cplusplus
}
#endif

#endif // ONDE_C_API_H