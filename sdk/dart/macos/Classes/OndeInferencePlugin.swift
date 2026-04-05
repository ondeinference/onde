// OndeInferencePlugin.swift
//
// Native macOS plugin for onde_inference.
//
// Responsibilities:
// 1. Provides a method channel ("com.ondeinference.onde_inference") so Dart
//    can resolve the App Group shared container path at runtime.
// 2. Acts as the CocoaPods framework host — the Rust static library
//    (libonde_inference_dart.a) is force-loaded into this framework via
//    OTHER_LDFLAGS in the podspec.

import FlutterMacOS
import Foundation

public class OndeInferencePlugin: NSObject, FlutterPlugin {

    /// The App Group identifier shared across all Onde-powered apps.
    /// Must match the value in DebugProfile.entitlements / Release.entitlements.
    private static let appGroupId = "group.com.ondeinference.apps"

    public static func register(with registrar: FlutterPluginRegistrar) {
        let channel = FlutterMethodChannel(
            name: "com.ondeinference.onde_inference",
            binaryMessenger: registrar.messenger
        )
        let instance = OndeInferencePlugin()
        registrar.addMethodCallDelegate(instance, channel: channel)
    }

    public func handle(
        _ call: FlutterMethodCall,
        result: @escaping FlutterResult
    ) {
        switch call.method {
        case "getAppGroupContainerPath":
            if let url = FileManager.default.containerURL(
                forSecurityApplicationGroupIdentifier: Self.appGroupId
            ) {
                result(url.path)
            } else {
                // App Group not configured or not entitled — fall back to nil
                // so the Dart side can use getApplicationSupportDirectory() instead.
                result(nil)
            }
        default:
            result(FlutterMethodNotImplemented)
        }
    }
}