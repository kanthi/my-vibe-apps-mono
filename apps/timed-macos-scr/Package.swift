// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "timed-macos-scr",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .library(
            name: "TimedClockCore",
            targets: ["TimedClockCore"]
        )
    ],
    targets: [
        .target(
            name: "TimedClockCore",
            path: "Sources/TimedClockCore"
        )
    ]
)
