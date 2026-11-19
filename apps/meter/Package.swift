// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "Meter",
    platforms: [.macOS(.v14)],
    targets: [
        .target(name: "MeterCore", path: "Sources/MeterCore"),
        .target(name: "MeterUI", dependencies: ["MeterCore"], path: "Sources/MeterUI"),
        .executableTarget(name: "Meter", dependencies: ["MeterUI"], path: "Sources/Meter"),
        .executableTarget(name: "meter-probe", dependencies: ["MeterCore"], path: "Sources/meter-probe"),
        .testTarget(name: "MeterCoreTests", dependencies: ["MeterCore"], path: "Tests/MeterCoreTests"),
        .testTarget(name: "MeterUITests", dependencies: ["MeterUI", "MeterCore"], path: "Tests/MeterUITests")
    ]
)
