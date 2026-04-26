// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.ondeinference.onde

import android.content.Context
import android.system.Os
import java.io.File

/**
 * Android implementation of [PlatformSupport].
 *
 * Uses `android.system.Os.setenv()` (requires API 26+) to seed environment
 * variables for the HuggingFace Hub cache. Native library loading is handled
 * automatically by the APK's `jniLibs/` directory — no extraction needed.
 */
internal object AndroidPlatform : PlatformSupport {
    override fun setEnv(key: String, value: String) {
        Os.setenv(key, value, true)
    }
    // ensureNativeLoaded() is intentionally a no-op on Android.
    // The UniFFI-generated companion object calls System.loadLibrary("onde"),
    // which loads the correct ABI from the APK's lib/ directory automatically.
}

/**
 * Create an [OndeInference] engine for Android.
 *
 * @param context Android application or activity context. Pass `applicationContext`
 *   to avoid leaking activity references.
 * @param dataDir Optional override for the root cache directory. Defaults to
 *   `context.filesDir`. Useful when the app wants to use external storage.
 * @return A ready-to-use [OndeInference] instance.
 */
fun OndeInference(
    context: Context,
    dataDir: File = context.filesDir,
): OndeInference = OndeInference(
    dataDir  = dataDir,
    platform = AndroidPlatform,
)
