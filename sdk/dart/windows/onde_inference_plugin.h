#ifndef FLUTTER_PLUGIN_ONDE_INFERENCE_PLUGIN_H_
#define FLUTTER_PLUGIN_ONDE_INFERENCE_PLUGIN_H_

#include <flutter/plugin_registrar_windows.h>

#include <memory>

namespace onde_inference {

// A minimal Flutter Windows plugin class.
//
// All on-device inference logic is implemented in the Rust crate
// onde_inference_dart (compiled to onde_inference_dart.dll).  This plugin
// class exists solely to satisfy the Flutter Windows plugin registration
// protocol — it does not handle any method channels itself.
//
// flutter_rust_bridge v2 loads the Rust DLL directly from Dart via dart:ffi,
// so no C++ ↔ Dart channel code is needed here.
class OndeInferencePlugin : public flutter::Plugin {
 public:
  static void RegisterWithRegistrar(flutter::PluginRegistrarWindows* registrar);

  OndeInferencePlugin();
  virtual ~OndeInferencePlugin();

  // Disallow copy and assign.
  OndeInferencePlugin(const OndeInferencePlugin&) = delete;
  OndeInferencePlugin& operator=(const OndeInferencePlugin&) = delete;
};

}  // namespace onde_inference

#endif  // FLUTTER_PLUGIN_ONDE_INFERENCE_PLUGIN_H_