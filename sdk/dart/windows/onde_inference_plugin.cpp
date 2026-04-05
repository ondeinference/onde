// Copyright 2024 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//
// onde_inference_plugin.cpp
//
// Minimal Flutter Windows plugin registration entry point.
//
// All LLM inference logic runs inside the Rust shared library
// (onde_inference_dart.dll) loaded at runtime by the flutter_rust_bridge Dart
// package.  This C++ file exists solely so that the Flutter tooling recognises
// onde_inference as a valid Windows plugin and links it into the host
// application.

#include "onde_inference_plugin.h"

// This must be included before many other Windows headers.
#include <windows.h>

#include <flutter/plugin_registrar_windows.h>

#include <memory>

namespace onde_inference {

// static
void OndeInferencePlugin::RegisterWithRegistrar(
    flutter::PluginRegistrarWindows* registrar) {
  auto plugin = std::make_unique<OndeInferencePlugin>();
  registrar->AddPlugin(std::move(plugin));
}

OndeInferencePlugin::OndeInferencePlugin() {}

OndeInferencePlugin::~OndeInferencePlugin() {}

}  // namespace onde_inference