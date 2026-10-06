// swift-tools-version: 6.0

import PackageDescription

let package = Package(
    name: "TetradUI",
    platforms: [
        .iOS(.v16),
        .macOS(.v13),
    ],
    products: [
        .executable(name: "TetradUI", targets: ["TetradUI"]),
    ],
    targets: [
        .executableTarget(
            name: "TetradUI",
            dependencies: ["TetradBridge"],
            path: "Sources/UI"
        ),
        .target(
            name: "TetradBridge",
            dependencies: ["TetradCore"],
            path: "Sources/Bridge"
        ),
        .systemLibrary(
            name: "TetradCore",
            path: "Sources/Core",
            pkgConfig: "tetrad",
        ),
    ]
)
