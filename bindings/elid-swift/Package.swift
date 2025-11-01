// swift-tools-version: 5.9
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "Elid",
    platforms: [
        .iOS(.v13),
        .macOS(.v10_15)
    ],
    products: [
        .library(
            name: "Elid",
            targets: ["Elid"]
        ),
    ],
    targets: [
        // Binary target for the Rust library
        // For local development, this uses a local path to the XCFramework
        // For distribution, replace with a remote URL and checksum
        .binaryTarget(
            name: "ElidFFI",
            path: "ElidFFI.xcframework"
        ),

        // Swift wrapper target
        .target(
            name: "Elid",
            dependencies: ["ElidFFI"],
            path: "Sources/Elid"
        ),

        // Test target
        .testTarget(
            name: "ElidTests",
            dependencies: ["Elid"],
            resources: [
                .copy("test-vectors.json")
            ]
        ),
    ]
)
