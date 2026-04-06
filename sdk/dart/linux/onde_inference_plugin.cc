// Copyright 2024 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license found in LICENSE.
//
// onde_inference_plugin.cc
//
// Minimal Linux Flutter plugin registration entry point for the
// onde_inference package.
//
// All real inference logic lives inside the Rust shared library
// (libonde_inference_dart.so) which is loaded at runtime by the
// flutter_rust_bridge Dart package.  This C++ file exists solely to satisfy
// the Flutter tooling's requirement that every Linux plugin provides a
// GObject-based plugin class registered via FlPluginRegistrar.

#include "include/onde_inference/onde_inference_plugin.h"

#include <flutter_linux/flutter_linux.h>
#include <gtk/gtk.h>

#define ONDE_INFERENCE_PLUGIN(obj) \
  (G_TYPE_CHECK_INSTANCE_CAST((obj), onde_inference_plugin_get_type(), \
                              OndeInferencePlugin))

struct _OndeInferencePlugin {
  GObject parent_instance;
};

G_DEFINE_TYPE(OndeInferencePlugin, onde_inference_plugin, G_TYPE_OBJECT)

// Called when the plugin object is being finalised.
static void onde_inference_plugin_dispose(GObject* object) {
  G_OBJECT_CLASS(onde_inference_plugin_parent_class)->dispose(object);
}

// Class initialiser.
static void onde_inference_plugin_class_init(
    OndeInferencePluginClass* klass) {
  G_OBJECT_CLASS(klass)->dispose = onde_inference_plugin_dispose;
}

// Instance initialiser.
static void onde_inference_plugin_init(OndeInferencePlugin* self) {}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

// Called by the Flutter engine when the plugin is registered with a
// FlPluginRegistrar.  Because all Flutter ↔ Rust communication is handled
// through the flutter_rust_bridge FFI layer (not through Flutter platform
// channels), no MethodChannel is registered here.
void onde_inference_plugin_register_with_registrar(
    FlPluginRegistrar* registrar) {
  OndeInferencePlugin* plugin = ONDE_INFERENCE_PLUGIN(
      g_object_new(onde_inference_plugin_get_type(), nullptr));

  g_object_unref(plugin);
}