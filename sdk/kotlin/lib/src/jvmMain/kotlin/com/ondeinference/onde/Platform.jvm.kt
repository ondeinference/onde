// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.ondeinference.onde

import java.io.File

/**
 * JVM (desktop) implementation of [PlatformSupport].
 *
 * Uses JNA to call the C library's `setenv()` function directly, since the
 * JVM standard library does not provide a way to modify environment variables
 * after process start. The Rust engine reads `HF_HOME` etc. via
 * `std::env::var()`, which reads from the C-level environment.
 *
 * Native library loading extracts the bundled `libonde.dylib` (macOS) or
 * `libonde.so` (Linux) from JAR resources and loads it via [System.load].
 */
internal object JvmPlatform : PlatformSupport {

    private interface CLib : com.sun.jna.Library {
        fun setenv(name: String, value: String, overwrite: Int): Int

        companion object {
            val INSTANCE: CLib = com.sun.jna.Native.load("c", CLib::class.java)
        }
    }

    override fun setEnv(key: String, value: String) {
        CLib.INSTANCE.setenv(key, value, 1)
    }

    override fun ensureNativeLoaded() {
        NativeLoader.ensureLoaded()
    }
}

/**
 * Create an [OndeInference] engine for JVM (desktop).
 *
 * On macOS, the default data directory is `~/.onde/`. On Linux it is also
 * `~/.onde/`. Model files are cached in `<dataDir>/models/hub/`.
 *
 * @param dataDir Root directory for the model cache and temp files.
 *   Defaults to `~/.onde/` in the user's home directory.
 * @return A ready-to-use [OndeInference] instance.
 */
fun OndeInference(
    dataDir: File = File(System.getProperty("user.home"), ".onde"),
): OndeInference = OndeInference(
    dataDir  = dataDir,
    platform = JvmPlatform,
)
