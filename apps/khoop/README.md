# Khoop

**An offline-by-default, on-device companion for WHOOP straps across iOS, macOS, and Android.**

Khoop is a local-first fitness companion that pairs directly with WHOOP straps (4.0, 5.0, and MG) over Bluetooth Low Energy, computes recovery, strain, HRV, sleep staging, and calorie deficit metrics locally, and stores all data in on-device SQLite.

- **Offline-by-default:** No external servers, no mandatory cloud accounts, no analytics or telemetry tracking.
- **Multidevice:** Native builds for **iOS (iPhone & Apple Watch)**, **macOS (Desktop & Menu Bar)**, and **Android (Compose & Room)**.
- **Fitness & Deficit Focus:** Calorie deficit tracking, food allowance calculations, workout expenditure against targets, weight-loss pacing, and biometric monitoring.

---

## Multidevice Architecture

```text
apps/khoop/
├── Packages/                   # Reusable Swift Packages (pure core)
│   ├── WhoopProtocol/          # BLE packet framing, CRC, event/command decoders
│   ├── WhoopStore/             # GRDB SQLite persistence & migrations
│   ├── StrandAnalytics/        # Recovery, strain, HRV, sleep, and correlation math
│   ├── StrandImport/           # Apple Health (export.xml) & WHOOP CSV importers
│   └── StrandDesign/           # SwiftUI design tokens, components, and charts
├── Strand/                     # macOS reference application & shared SwiftUI code
├── StrandiOS/                  # iOS application shell + Cut weight-loss/deficit features
├── StrandiOSShared/            # Shared code across iOS targets and extensions
├── StrandiOSWidgets/           # iOS Lock Screen & Home Screen widgets, Live Activities
├── KhoopWatch/                  # Apple Watch companion app (watchOS)
├── KhoopWatchComplications/     # Watch face complication widgets
├── android/                    # Android native app (Kotlin, Jetpack Compose, Room)
├── project.yml                 # XcodeGen project definition for Apple targets
└── Config/                     # Build and Bundle ID configuration
```

---

## Platform Features

### 1. iOS (iPhone & Apple Watch)
- **Cut Today View:** Daily calorie allowance ring, fat deficit estimate, heart rate, steps, and strap battery level.
- **Food & Nutrition Log:** Rapid calorie and protein logging with recent food item memory.
- **Sleep & Recovery:** Multi-stage sleep hypnograms, resting heart rate, and overnight HRV.
- **Live Activities & Widgets:** Dynamic Island status, lock screen workout banners, and widget complications.
- **Apple Watch:** Companion wrist app displaying charge, effort, and live heart rate.

### 2. macOS (Desktop & Menu Bar)
- **Full Desktop Dashboard:** Multi-pane dashboard with detailed sparklines, recovery scores, and trend graphs.
- **Menu Bar Extra:** Glanceable live heart rate, battery percentage, and quick sync popover in the system menu bar.
- **Historical Analysis & Lab Book:** Deep biometric history, n-of-1 correlation engine, and metric explorer.
- **Offline AI Coach (Optional):** Bring-Your-Own-Key local coach (supporting Ollama / LM Studio on `localhost` or remote LLM providers) with strict consent gating.

### 3. Android
- **Native Android Experience:** Built using modern Jetpack Compose, Kotlin Coroutines/Flow, and Room SQLite database.
- **Full Parity:** Mirror implementation of the protocol decoders, scoring algorithms, and background sync services.
- **No Cloud Dependency:** Operates completely standalone via Bluetooth background services.

---

## Build Instructions

### Prerequisites
- **macOS / iOS:** Xcode 15+ and [XcodeGen](https://github.com/yonaskolb/XcodeGen) (`brew install xcodegen`).
- **Android:** JDK 17 and Android SDK 34 (`platform-tools`, `platforms;android-34`).

### 1. Building for macOS
```bash
# Generate the Xcode project
xcodegen generate

# Build from terminal or open in Xcode
xcodebuild -project Strand.xcodeproj -scheme Strand -destination 'platform=macOS' CODE_SIGNING_ALLOWED=NO build
```
Or open `Strand.xcodeproj` in Xcode, select the **Strand** scheme with destination **My Mac**, and press **Cmd + R**.

### 2. Building for iOS
```bash
# 1. Configure your bundle ID prefix and development team (optional for simulator, required for device):
cp Config/BundleIdSecrets.example.xcconfig Config/BundleIdSecrets.xcconfig
# Edit BUNDLE_ID_PREFIX and DEVELOPMENT_TEAM in Config/BundleIdSecrets.xcconfig

# 2. Generate the project:
xcodegen generate

# 3. Open in Xcode:
open Strand.xcodeproj
```
Select the **KhoopIOS** scheme and your target iOS device or simulator.

### 3. Building for Android
```bash
cd android

# Run unit tests:
./gradlew testFullDebugUnitTest

# Assemble debug APK:
./gradlew assembleFullDebug
```
The output APK is generated at `android/app/build/outputs/apk/full/debug/app-full-debug.apk`.

---

## Security & Privacy

Khoop adheres to strict local-first security standards:
- **Zero Telemetry:** No analytics libraries, no third-party tracking pixels, and no phoning home.
- **Secure Credentials:** API keys for optional features (e.g. AI Coach) are stored exclusively in the system Keychain (`kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly`).
- **Hardware Safe BLE:** The Bluetooth engine implements a strictly curated, non-destructive subset of strap opcodes. Destructive commands (firmware modification, DFU, trim, or factory wipe) are excluded at the schema level.
- **Local Storage:** All metrics and recordings remain on the user's physical device in SQLite.

---

## License & Attribution

Khoop is based on [Khoop](https://github.com/ryanbr/khoop) and [Zhoop](https://github.com/hackyguru/khoop).
Distributed under the [PolyForm Noncommercial 1.0.0](LICENSE) license.

WHOOP is a trademark of WHOOP, Inc. Khoop is an independent clean-room interoperability project and is not affiliated with, sponsored by, or endorsed by WHOOP, Inc.
