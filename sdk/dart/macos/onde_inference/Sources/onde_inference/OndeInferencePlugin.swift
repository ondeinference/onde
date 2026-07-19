// Copyright 2026 Splitfire AB (Onde Inference). All rights reserved.
// SPDX-License-Identifier: MIT OR Apache-2.0

import FlutterMacOS
import Foundation

public class OndeInferencePlugin: NSObject, FlutterPlugin {
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
                result(nil)
            }
        default:
            result(FlutterMethodNotImplemented)
        }
    }
}
