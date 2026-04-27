// Copyright 2025 Onde Inference (Splitfire AB). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.ondeinference.onde

internal interface PlatformSupport {
    fun setEnv(key: String, value: String)
    fun ensureNativeLoaded() {}
}
