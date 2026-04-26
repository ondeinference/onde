// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.ondeinference.onde

/**
 * Platform-specific operations required by [OndeInference].
 *
 * Each platform target provides an implementation:
 * - Android: [AndroidPlatform] — uses `android.system.Os.setenv`
 * - JVM: [JvmPlatform] — uses JNA to call libc `setenv`
 */
internal interface PlatformSupport {
    /**
     * Set a process-level environment variable.
     *
     * This is used to configure `HF_HOME`, `HF_HUB_CACHE`, `TMPDIR` etc.
     * before any Rust/HuggingFace Hub operation.
     */
    fun setEnv(key: String, value: String)

    /**
     * Ensure the native `libonde` library is loaded.
     *
     * On Android this is a no-op — `System.loadLibrary("onde")` is handled
     * by the UniFFI-generated companion object init block, which loads from
     * the APK's `jniLibs/` directory automatically.
     *
     * On JVM this extracts the bundled `.dylib`/`.so` from JAR resources
     * and calls `System.load(path)` before UniFFI's init can run.
     */
    fun ensureNativeLoaded() {}
}
