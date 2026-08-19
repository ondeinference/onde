// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "onde_inference",
    platforms: [
        .macOS("12.0")
    ],
    products: [
        .library(name: "onde-inference", targets: ["onde_inference"])
    ],
    dependencies: [],
    targets: [
        .target(
            name: "onde_inference",
            path: "Sources/onde_inference"
        )
    ]
)
