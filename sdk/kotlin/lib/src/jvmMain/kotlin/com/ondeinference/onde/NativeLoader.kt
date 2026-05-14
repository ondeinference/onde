// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.ondeinference.onde

import java.io.File
import java.io.FileOutputStream

/**
 * Extracts and loads the native `libonde` library from JAR resources.
 *
 * The JVM target bundles native libraries under:
 *   `/native/<os>-<arch>/libonde.<ext>`
 *
 * Supported platforms:
 *   - `macos-aarch64/libonde.dylib` — macOS Apple Silicon
 *   - `macos-x86_64/libonde.dylib`  — macOS Intel
 *   - `linux-x86_64/libonde.so`     — Linux x86_64
 *   - `linux-aarch64/libonde.so`    — Linux ARM64
 *
 * If the native library is not bundled (e.g. development mode), falls back to
 * [System.loadLibrary] which searches `java.library.path`.
 */
internal object NativeLoader {
    @Volatile private var loaded = false

    fun ensureLoaded() {
        if (loaded) return
        synchronized(this) {
            if (loaded) return
            try {
                loadFromResources()
            } catch (_: Exception) {
                // Fallback: library must be on java.library.path
                System.loadLibrary("onde")
            }
            loaded = true
        }
    }

    private fun loadFromResources() {
        val osName = System.getProperty("os.name").lowercase()
        val archName = System.getProperty("os.arch").lowercase()

        val (os, ext) = when {
            "mac" in osName || "darwin" in osName -> "macos" to "dylib"
            "linux" in osName                     -> "linux" to "so"
            "windows" in osName                   -> "windows" to "dll"
            else -> throw UnsupportedOperationException(
                "Onde Inference does not support this OS: $osName"
            )
        }

        val arch = when (archName) {
            "aarch64", "arm64"        -> "aarch64"
            "x86_64", "amd64"        -> "x86_64"
            else -> throw UnsupportedOperationException(
                "Onde Inference does not support this architecture: $archName"
            )
        }

        val resourcePath = "/native/$os-$arch/libonde.$ext"
        val stream = NativeLoader::class.java.getResourceAsStream(resourcePath)
            ?: throw UnsatisfiedLinkError(
                "Native library not found in JAR resources: $resourcePath. " +
                "Run scripts/build-jvm.sh to build and bundle the native library, " +
                "or ensure libonde is on java.library.path."
            )

        val tempDir = File(System.getProperty("java.io.tmpdir"), "onde-native")
        tempDir.mkdirs()
        val tempFile = File(tempDir, "libonde.$ext")

        // Overwrite on every load to handle version upgrades
        stream.use { input ->
            FileOutputStream(tempFile).use { output ->
                input.copyTo(output)
            }
        }

        System.load(tempFile.absolutePath)
    }
}
