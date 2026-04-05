// This file exists so CocoaPods creates a real framework target for the
// onde_inference pod.  The actual native code is the Rust static library
// (libonde_inference_dart.a) which is force-loaded into this framework
// via OTHER_LDFLAGS in the podspec.
//
// Do not add any logic here.

import Flutter

public class OndeInferencePlugin: NSObject, FlutterPlugin {
    public static func register(with registrar: FlutterPluginRegistrar) {
        // No-op — all FFI symbols come from the Rust static library.
    }
}
