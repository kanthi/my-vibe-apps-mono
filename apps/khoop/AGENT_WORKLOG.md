# AGENT_WORKLOG.md — Khoop Ongoing Agent Context & Work Log

> [!IMPORTANT]
> **MANDATORY INSTRUCTION FOR ALL AI AGENTS:**
> 1. **Read this file FIRST** before touching any code in `apps/khoop/`.
> 2. **Update this file BEFORE completing your turn** whenever you make changes, discover key findings, run builds, or advance tasks. Record what was done, files touched, build/test results, and next steps so the user never has to re-explain project context.

---

## 1. Project Overview & Hard Constraints

Khoop is an **offline-by-default, on-device** companion app for **WHOOP 4.0 and 5.0/MG straps** (plus experimental Oura decoding).
- **Core Principles:**
  - **Zero Telemetry / Zero Cloud:** No backend server, no accounts, no analytics libraries, no tracking pixels. Operates 100% locally.
  - **Local Storage:** All biometric metrics (HRV, recovery, strain, sleep hypnograms, deficit calculations) are stored in on-device SQLite databases.
  - **Cross-Platform:**
    - macOS & iOS: Written in Swift (`Strand/`, `StrandiOS/`, `Packages/`).
    - Android: Written in Kotlin with Jetpack Compose & Room (`android/`).
  - **Clean-room BLE Interoperability:** Implements non-destructive strap opcodes only. Hardware safe.

---

## 2. Android Build System & Toolchain Reference

### Environment Requirements
- **JDK:** OpenJDK 17.
  - macOS path: `/opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk/Contents/Home` (or Android Studio JBR at `/Applications/Android Studio.app/Contents/jbr/Contents/Home`).
- **Android SDK:** Installed at `/Users/king/Library/Android/sdk` (configured via `android/local.properties`).
  - Target SDK: 34 (API 34), Compile SDK: 35, Min SDK: 26.
  - Build Tools: `34.0.0` or `36.0.0`.
- **Gradle:** Wrapper pins Gradle **8.7** with Android Gradle Plugin (AGP) **8.5.2**.

### Product Flavors & Build Types
- **Flavors:**
  - `full`: The real Khoop app (`applicationId = "com.khoop.app"`, `ENABLE_DEMO = false`). Connects to real WHOOP BLE hardware and stores genuine strap telemetry.
  - `demo`: "Khoop Demo" (`applicationId = "com.khoop.app.demo"`, `ENABLE_DEMO = true`). Preloaded with 120 days of synthetic data for UI preview without a strap.
- **Build Types:**
  - `release`: Signed using `android/release.keystore` via credentials in `android/keystore.properties`. Unminified for runtime reflection stability.
  - `debug`: Signed using debug keystore, appends `.debug` suffix to package ID.
- **Gradle Build Commands:**
  ```bash
  cd apps/khoop/android
  export JAVA_HOME="/opt/homebrew/opt/openjdk@17/libexec/openjdk.jdk/Contents/Home"
  
  # Build only the real release APK (recommended):
  ./gradlew assembleFullRelease
  
  # Clean & rebuild:
  ./gradlew clean assembleFullRelease
  
  # Run JVM unit tests:
  ./gradlew testFullDebugUnitTest
  ```

### Release vs Full Release vs Debug Explanation
- **`assembleFullRelease` vs `assembleRelease`:**
  - `assembleFullRelease`: Builds specifically the real production `fullRelease` APK.
  - `assembleRelease`: Gradle lifecycle task that builds release APKs for **both** `full` and `demo` flavors.
- **Updating an Installed App:**
  - A device with the release APK already installed **must be updated with the `release` APK** (`fullRelease`), signed by `release.keystore`.
  - **Do NOT install `-debug.apk`** over an existing release app; Android rejects it due to signature mismatch and different package ID (`com.khoop.app.debug`).

### Build Artifact Locations
- **Gradle Outputs:** `apps/khoop/android/app/build/outputs/apk/full/release/khoop-v<version>-<date>-fullRelease.apk`
- **Archived Build Artifacts:** `apps/khoop/build-artifacts/`
  - `khoop-v0.007-<date>-release.apk`: Versioned release APK.
  - `khoop-full-release.apk`: Static unversioned alias of the release APK for quick distribution / scripts.
  - `khoop-v0.007-<date>-debug.apk`: Debug APK.
  - `khoop-full-debug.apk`: Static unversioned debug APK.

---

## 3. Work Log & Change History

### [2026-10-01] — Bevel feature and UI reference (no code)

- **Work performed:** Compared public Bevel (bevel.health, App Store id 6456176249, help center) with Khoop. Saved the writeup and captured UI under the private clone at `/Users/king/Workspace/Repos/khoop/references/bevel/`. No old-tree source was edited. No Bevel feature was adopted into the new app. The running Android slice and the next widget-typeface pass are unchanged.
- **Build / tests:** None. This was a reference pass.
- **Next:** Widget text face on the new app, as already noted in the private clone's `CONTINUE.md`.

### [2026-09-29] — Release v0.007 APK Build & Android Enhancements
- **Work Performed:**
  - User added and refined Glance widgets:
    - [`HrGlanceWidget.kt`](android/app/src/main/java/com/khoop/widget/HrGlanceWidget.kt)
    - [`KhoopGlanceWidget.kt`](android/app/src/main/java/com/khoop/widget/KhoopGlanceWidget.kt)
    - [`StressGlanceWidget.kt`](android/app/src/main/java/com/khoop/widget/StressGlanceWidget.kt)
    - [`KhoopCompactGlanceWidget.kt`](android/app/src/main/java/com/khoop/widget/KhoopCompactGlanceWidget.kt)
    - [`CoachBriefGlanceWidget.kt`](android/app/src/main/java/com/khoop/widget/CoachBriefGlanceWidget.kt)
  - Recovery drivers and analytics updates:
    - [`RecoveryDrivers.kt`](android/app/src/main/java/com/khoop/analytics/RecoveryDrivers.kt)
    - [`RecoveryDriversTest.kt`](android/app/src/test/java/com/khoop/analytics/RecoveryDriversTest.kt)
  - UI palette tokens & Theme updates:
    - [`PaletteTokens.kt`](android/app/src/main/java/com/khoop/ui/PaletteTokens.kt)
    - [`Theme.kt`](android/app/src/main/java/com/khoop/ui/Theme.kt)
    - [`SettingsScreen.kt`](android/app/src/main/java/com/khoop/ui/SettingsScreen.kt)
  - Unit tests & crash capture assertions:
    - [`CrashCaptureTest.kt`](android/app/src/test/java/com/khoop/CrashCaptureTest.kt)
    - [`LogExportCrashSectionTest.kt`](android/app/src/test/java/com/khoop/ui/LogExportCrashSectionTest.kt)
    - [`HealthConnectSelfWriteSkipTest.kt`](android/app/src/test/java/com/khoop/ingest/HealthConnectSelfWriteSkipTest.kt)
  - Clean release compilation:
    - Executed `./gradlew clean assembleFullRelease` with JDK 17.
    - Verified APK signature with `apksigner verify --verbose` (v2 Signature Scheme valid, signer `CN=Khoop`).
    - Synced output APK to `apps/khoop/build-artifacts/khoop-v0.007-20260929-release.apk` and `khoop-full-release.apk`.
  - Demo release build & Emulator Verification:
    - Compiled `./gradlew assembleDemoRelease` generating `khoop-v0.007-demo-20260929-demoRelease.apk`.
    - Booted Android virtual device `Medium_Phone` (API 34/35) and deployed both `full` and `demo` variants.
    - Verified UI/UX rendering:
      - Validated that `NothingTokens` (pure OLED `#000000` base, Nothing Red `#D71920` pills, buttons, and accents) activate cleanly when in dark mode.
      - Confirmed that the first-run Terms gate, onboarding steps (1–12), and More tab drawer icons properly reflect the Nothing OS theme tokens.
    - Gracefully shut down and killed all emulator / QEMU processes after testing.
  - Added Agent Logging Protocol to root `AGENTS.md` and created this `AGENT_WORKLOG.md`.

---

### [2026-09-30] — Defensive security audit (report only, no code changes)

Finished the audit that was started on the Zhoop checkout and interrupted before a write-up. Re-checked the findings against `apps/khoop` (package `com.khoop`, not `com.noop`). No source files were modified.

**Findings to treat as real:**

1. **Release signing material is on disk next to the source, mode `644`.** `android/release.keystore` and `android/keystore.properties` (plaintext store password) are gitignored and not tracked. `android/fork-debug.keystore` is gitignored and mode `664`. `build.gradle.kts` hardcodes the debug store password `android`. A release-signed install uses application id `com.khoop.app`. Anyone who can read this folder can sign an update Android will accept over that install, keeping the health database and the Keystore key that decrypts AI / Oura / push credentials. Debug builds use `applicationIdSuffix = ".debug"`, so they do not replace the release app.
2. **Android cleartext is global, and the AI client follows redirects.** `network_security_config.xml` sets `cleartextTrafficPermitted="true"`. `AiCoach` builds OkHttp 4.12 with timeouts only (no `followRedirects(false)`). `guardCustomUrl` / `isPrivateLanOrLoopback` run on the typed URL only. `PushHttpTransport` disables both redirect flags. Apple `NSAppTransportSecurity` is `NSAllowsLocalNetworking` only (`project.yml` and both Info.plists). `URLSession.shared` still follows redirects with no second host check. OkHttp's `RetryAndFollowUpInterceptor` class contains the string `Authorization` and does not contain `x-api-key` (Anthropic and the optional custom header). Not decompiled.
3. **Update Download opens `html_url` with no allowlist.** Android `SettingsScreen` `ACTION_VIEW`s `avail.url`; Apple `SettingsView` calls `openURL(url)`. Endpoints disagree: Android `UpdateCheck` uses `api.github.com/repos/ryanbr/khoop/releases/latest`; `UpdateChecker.swift` uses `hackyguru/khoop`. `docs/PRIVACY_SECURITY.md` says both use `ryanbr/khoop`. `HttpURLConnection` and `URLSession.shared` follow redirects by default.

**Controls that held on re-read:** `allowBackup=false` plus extraction rules; FileProvider `exported=false` and cache-only paths; widget/launcher exports are the required ones; connection service, notification listener, phone/alarm receivers `exported=false`; phone receiver reads call state only; notification listener uses package/category/ongoing and `VoipCallClassifier` avoids extras; AI keys in `EncryptedSharedPreferences` / Keychain and provider-scoped; push default off, wifi-only default, bearer in `SecurePrefs`, HTTP only for a literal local address, no DNS during validation; zip restore matches basename to fixed dest files with a size cap; `khoop://import-health` stages and confirms before writing `apple-health` only; Oura OAuth checks `state`; Oura secrets are example placeholders, real xcconfig gitignored; no WebView; no HTTP logging interceptor; DFU/firmware opcodes are decode names plus the ECG deny-list, not a send surface. `REBOOT_STRAP` is a separate user-confirmed action. `khoop-local-access` is stdio, read-only, no socket.

**Not fully re-audited:** every BLE write site beyond the sender enum and ECG allow-list; git history for a secret later removed; OkHttp redirect branch logic beyond the class strings.

### [2026-09-30] — Private GitHub repo created; code not pushed yet

- Created empty private repo https://github.com/kanthi/khoop (no README, no initial commit, wiki off).
- `apps/khoop/` is still untracked in my-vibe-apps-mono (`?? apps/khoop/`). There is no nested `.git` and no monorepo history to extract with `git filter-repo`.
- A first commit of the gitignore-respecting tree is about 3,053 files / 79 MB. Ignored and must stay local: `release.keystore`, `fork-debug.keystore`, `keystore.properties`, `local.properties`, `android/app/build` (~701 MB), `build-artifacts` APKs (~102 MB).
- Next step is the migration plan agreed with the user: new working copy at `~/Workspace/Repos/khoop`, one initial commit, push to `kanthi/khoop`, then replace this folder with a pointer. Not done in this turn.

### [2026-09-30] — Product map written; empty repo cloned locally, still no source copy

Studied the tree before any copy. Full notes for later planning: `~/.grok/memory-v2/global/topics/khoop-existing-app.md`.

What this folder actually is:

- Offline WHOOP 4.0 / 5.0 / MG companion. BLE → SQLite → local Charge / Effort / Rest scores. No Khoop account or server. Not a medical device. PolyForm Noncommercial. Snapshot of ryanbr/khoop and hackyguru/khoop, version string `0.007`, **untracked** in the incubator (no `.git` here).
- Apple: `Packages/` (protocol, GRDB store, analytics, import, design) + `Strand/` macOS reference (full sidebar) + `StrandiOS/` + widgets + watch. Android: independent Kotlin/Compose/Room full app (`com.khoop.app`), flavors `full` and `demo`. Analytics are meant to match across platforms.
- **iOS-only fork cut:** `RootTabView` shows two tabs. Today is `Cut/CutTodayView` (calorie budget, food log, goal weight, live HR, battery, burn, steps). Sleep is `Cut/CutSleepView`. The plan and food log live in UserDefaults (`CutPlanStore`), not `whoop.sqlite`. Android and macOS still ship the full Khoop shell (Today, Trends, Sleep, Coach, More, and the rest). The README's "calorie deficit tracker" sentence describes that iOS cut, not the whole tree.
- Optional network: AI Coach (bring-your-own key), Android experimental one-way push, daily GitHub update check (Android `ryanbr/khoop`, Apple `hackyguru/khoop`), optional Oura OAuth.
- Gitignored and must not be pushed: keystores, `keystore.properties`, `local.properties`, `android/app/build`, APKs under `build-artifacts`.

### [2026-09-30] — New repo: Android-only sample day, more strap detail, 4a Pro screenshots

Work is in `/Users/king/Workspace/Repos/khoop`, not in this incubator tree. Nothing here was edited for the new app. Nothing in the private clone is committed.

- User clarified the two UI references are a design language. The product for now is an Android app on an Android device. Screenshots are `adb screencap` on AVD `Khoop_4a_Pro` (1260×2800, 440 dpi), with Android 3-button navigation.
- Sample day on Today, Sleep, and Heart now shows more strap-backed rows: strap card (respiration, SpO2 range, skin range, mean R-R, battery), sleep stage breakdown, and heart-rate time in range. Still a labeled sample. No BLE, no SQLite, no widgets.
- Shots to use: `references/emulator/android-today-more.png`, `android-sleep-more.png`, `android-heart-more.png`. `android-today.png` is a bad onboarding frame from a navigation-mode change.
- Build that succeeded earlier: JDK 17 at `/opt/homebrew/opt/openjdk@17`, `./gradlew :app:assembleDebug`, application id `com.khoop`, version `0.1.0`. Emulator was shut down with `adb emu kill` after the shots.
- Next: keep appending the user's notes to `plan.md`. Do not commit until they say so. Do not copy this old tree into the new repo.
- 2026-09-30 later: user asked about widgets, then said to put the three on the home screen. Built in the private clone, not in this tree: 1×1 Charge, 1×1 heart rate (sample 66), 2×1 Today. A follow-up the same night replaced the transparent number row with rounded monochrome tiles: Charge has Steady and a pastel meter, Heart has bpm and the day's trace, Today has "Charge is steady." plus Charge, Rest, and Effort 13 of 21. Verified on AVD `Khoop_4a_Pro` (1260×2800, 440 dpi), shot `references/emulator/android-widgets-home.png`. User said the tiles are a lot better and the next pass is the right typeface on the widget text. Dotted numerals stay the dot grid. Continue tomorrow. Lock screen and wallpaper tint still open. Not committed.
- 2026-09-30 end of day: user asked to save the discussion in the Khoop repo. Handoff is `/Users/king/Workspace/Repos/khoop/CONTINUE.md`. Read that before the next session. `plan.md` remains the running log. Nothing committed.

## 4. Ongoing Work & Next Steps

5. **Spin-out:** https://github.com/kanthi/khoop is private. Local clone `/Users/king/Workspace/Repos/khoop` has two uncommitted planning files only: `features.md` and `plan.md`. The user said planning continues until they finalize it, and no application code is to be written before that. Primary target recorded in the new repo's `plan.md`: Android on Nothing OS, Nothing Phone (4a) Pro (6.83", 1260×2800, 20:9), layout and resolution first. Visual rule: look like a default Nothing OS app; pastel only on graphs, rings, and similar data marks. Widgets: Nothing OS built-in style (monochrome, dot-matrix, 1×1 and 2×1 included). Which metrics they show is still open. Logo and launcher icon are required, Nothing OS style, mark not chosen, not drawn during planning. Settings will offer Light and Dark, both in the Nothing monochrome. Follow-system is not decided. Collection rule: store every non-destructive reading the paired strap streams or offloads, including signals with no screen yet; firmware, DFU, and wipe stay out. Phone sensors and Health Connect were not part of that note. Visual target is now Nothing OS 5 (4a Pro stable early November 2026): Geist via the system type, dot-matrix as accent, monochrome surfaces, optional Lucent transparency on widgets, wallpaper tint left open. First-run onboarding, including the terms gate, uses that same native style; the old 12-page list is the content reference and which pages stay is open. Prometheus export was asked about and not adopted: optional summary push to a user-run server, SQLite stays the archive. UI baseline is the two attached health screens, copied to `/Users/king/Workspace/Repos/khoop/references/`; layout kept, restyle toward Nothing OS 5 on the 4a Pro. Body age, body fat, muscle score, and cardio fitness are reference-only until decided. The new app is a from-scratch Android codebase in the private repo. The old tree stays a reference and is not copied in. No application code until the plan is finalized. Do not commit or push the private clone until the user explicitly says so. First native slice written 2026-09-30 in `/Users/king/Workspace/Repos/khoop`: Kotlin, Compose, application id `com.khoop`, sample day, Nothing-style onboarding plus Today, Sleep, Heart, and Settings. Checked on AVD `Khoop_4a_Pro` (1260×2800, 440 dpi) in Light and Dark. No strap protocol, no SQLite, no widgets, no commit. 2026-09-30 later the same day: references are design language only; the app stays Android-only for now; more strap detail (stage rows, heart-rate zones, R-R, SpO2 and skin ranges) was added and re-shot on `Khoop_4a_Pro`. Use `references/emulator/android-today-more.png`, `android-sleep-more.png`, and `android-heart-more.png`. Append their notes to `plan.md`. Do not copy this tree, do not commit, and do not `git filter-repo`.



5. **Spin-out:** private repo is https://github.com/kanthi/khoop (empty). Push a clean snapshot from a new working copy; do not use filter-repo. Leave signing keys on disk only.


1. **Hardware BLE Verification:**
   - Real device validation with WHOOP 4.0 and 5.0 straps over BLE (handshake bond, type-47 historical offload, live HR streaming).
2. **Health Connect Sync:**
   - Verify non-destructive import/export and self-write skip handling.
3. **Parity Maintenance:**
   - Verify byte-for-byte analytics and schema parity with Swift packages in `Packages/` and `Strand/`.
4. **Subsequent Agent Requirement:**
   - Whenever any agent updates code or builds new packages in `apps/khoop/`, append a new entry to Section 3 above detailing what was changed and the current build state.

## 2026-11-14: .gitignore hardened

Added ignore rules so the app stays clean after extraction from the incubator (repo-wide ignore pass). Added `build-artifacts/` (102 MB, was only ignored at the repo root) and `marketing/*.mp4` (about 25 MB of rendered demo videos; regenerate from `marketing/Khoop-demo.source.html`). No code changes.
