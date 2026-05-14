// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//
// OndeInferencePlugin.swift
//
// Native iOS plugin for onde_inference.
//
// Responsibilities:
// 1. Provide a method channel ("com.ondeinference.onde_inference") so Dart
//    can resolve the App Group shared container path at runtime.
// 2. Act as the CocoaPods framework host. The Rust static library
//    (libonde_inference_dart.a) is force-loaded into this framework through
//    OTHER_LDFLAGS in the podspec.

import Flutter
import UIKit

public class OndeInferencePlugin: NSObject, FlutterPlugin {

    /// The App Group identifier shared across all Onde-powered apps.
    /// Must match the value in Runner.entitlements.
    private static let appGroupId = "group.com.ondeinference.apps"

    public static func register(with registrar: FlutterPluginRegistrar) {
        let channel = FlutterMethodChannel(
            name: "com.ondeinference.onde_inference",
            binaryMessenger: registrar.messenger()
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
                // If the App Group is missing or not entitled, return nil.
                // Dart can fall back to getApplicationSupportDirectory() instead.
                result(nil)
            }
        default:
            result(FlutterMethodNotImplemented)
        }
    }
}
