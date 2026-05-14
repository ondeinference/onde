// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.

import Flutter
import UIKit

public class OndeInferencePlugin: NSObject, FlutterPlugin {
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
                result(nil)
            }
        default:
            result(FlutterMethodNotImplemented)
        }
    }
}
