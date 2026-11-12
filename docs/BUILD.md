# Building WordCraft (Baell) for Android

This document explains how the Android build works and how to build it locally if you wish.

## How it works

The Android port has two parts:

1. **The Rust crate** (`android/`): the `wordcraft-android` cdylib. It reuses the upstream
   WordCraft crates (`wordcraft-engine`, `wordcraft-ui-egui`, …) verbatim and only adds the
   Android host: `android_main`, an external winit event loop for eframe, and the platform
   services a phone can offer (the built-in file browser instead of native file dialogs,
   downloads into the app's own data directory, recovery, user fonts and dictionary).
   See `android/src/lib.rs` and `android/src/host.rs`.

2. **The Gradle project** (`android/`): a plain `android.app.NativeActivity` host. There is no
   Java or Kotlin code. Gradle only packages the `libwordcraft_android.so` libraries from
   `app/src/main/jniLibs/<abi>/` into APKs; it never calls the Rust toolchain.

The libraries are built with [cargo-ndk](https://crates.io/crates/cargo-ndk) from the
**workspace root**, where `[patch.crates-io]` pins egui, eframe and winit to the WordCraft
forks. cargo-ndk maps ABI names to Rust targets:

| ABI | Rust target |
|---|---|
| `arm64-v8a` | `aarch64-linux-android` |
| `armeabi-v7a` | `armv7-linux-androideabi` |
| `x86_64` | `x86_64-linux-android` |

### Build toolchain

| Tool | Version |
|---|---|
| Rust | stable (≥ 1.95, the workspace's `rust-version`) |
| Android NDK | r27 (27.0.12077973) |
| Gradle | 8.10.2 |
| Android Gradle Plugin | 8.5.2 |
| JDK | 17 |
| `compileSdk` / `targetSdk` / `minSdk` | 35 / 35 / 24 |

There is no committed Gradle wrapper; install Gradle 8.10.2 (or run `gradle wrapper
--gradle-version 8.10.2` once to generate one).

## GitHub Actions (no local toolchain needed)

`.github/workflows/ci-android.yml`:

- On every push: `cargo fmt --check`, `cargo check` and `cargo clippy -D warnings` for
  `wordcraft-android` against the `aarch64-linux-android` target (compiles the whole front end
  for Android without linking), then debug APKs for all three ABIs, uploaded as workflow
  artifacts.
- On a tag matching `v*-android*` (say `v0.18.1-android.1`): the same in release mode, signed
  APKs (one per ABI plus a universal one) and an AAB, published on a GitHub release.

### Release signing (optional)

By default the release APKs are signed with a key generated on the runner — fine for testing
installs, but not a stable identity for updates. For a stable one, add these repository
secrets (`Settings ▸ Secrets and variables ▸ Actions`):

- `ANDROID_KEYSTORE_B64` — a base64 encoding of your `.jks`/`.keystore` file
  (`base64 -w0 keystore.jks`)
- `ANDROID_KEYSTORE_PASSWORD`
- `ANDROID_KEY_ALIAS`
- `ANDROID_KEY_PASSWORD`

Keep the keystore file itself out of the repository.

## Local builds

With the toolchain above installed (Rust with the Android targets, the NDK, Gradle, a JDK):

```bash
# From the repository root (the workspace root, so the [patch.crates-io] forks apply):
cargo ndk \
  -t arm64-v8a -t armeabi-v7a -t x86_64 \
  -o android/app/src/main/jniLibs \
  build --release -p wordcraft-android

# Package and install (from android/):
gradle :app:assembleDebug
adb install -r app/build/outputs/apk/debug/app-arm64-v8a-debug.apk
```

For release APKs and an AAB, add the injected signing properties:

```bash
gradle :app:assembleRelease :app:bundleRelease \
  -Pandroid.injected.signing.store.file=/path/to/keystore.jks \
  -Pandroid.injected.signing.store.password=… \
  -Pandroid.injected.signing.key.alias=… \
  -Pandroid.injected.signing.key.password=…
```

Outputs land in `android/app/build/outputs/apk/release/` (with ABI splits: one APK per ABI
plus `app-universal-release.apk`) and `android/app/build/outputs/bundle/release/`.

Optional build inputs, as for the desktop app: `CRAFT_I18N_DIR` (interface translations and
spelling/hyphenation data, pinned by `craft-i18n.rev`) and `CRAFT_FONTS_DIR` (extra fonts).
Without them the build works and the interface is English only.
