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

### Release signing (required for releases)

Every APK and the AAB is signed with one stable key, so Android installs a new release over the
previous one. The `release` job refuses to publish without the key rather than shipping a
throwaway signature that would force an uninstall for every update.

Create the key once, outside the repository:

```powershell
pwsh packaging/android/generate-signing-key.ps1
```

It writes `baell-release.keystore` and `baell-release.secrets.txt` (the four values below) under
`~/.wordcraft/android/`. Back both up: the keystore cannot be recovered. Then add each value under
`Settings ▸ Secrets and variables ▸ Actions`:

- `ANDROID_KEYSTORE_B64` — the base64 encoding of the `.jks`/`.keystore` file
- `ANDROID_KEYSTORE_PASSWORD`
- `ANDROID_KEY_ALIAS` — `baell-release` by default
- `ANDROID_KEY_PASSWORD`

Never commit the keystore or the secrets file. To use an existing key instead, encode it yourself
(`[Convert]::ToBase64String([IO.File]::ReadAllBytes('key.keystore'))`) and add the same four
secrets.

### Release version

The tag is the one source of truth: `v0.18.1-android.1` becomes versionName `0.18.1-android.1`
and versionCode `180101`. versionCode must strictly increase, so use a higher build number
(`v0.18.1-android.2`) for a follow-up release that does not change the upstream version.
`android/app/build.gradle` keeps fallback values for local builds; the workflow overrides them
through `BAELL_VERSION_NAME` and `BAELL_VERSION_CODE`.

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

For release APKs and an AAB, add the injected signing properties, and set the same version the
GitHub release uses so a local build installs over (not under) a published one:

```bash
export BAELL_VERSION_NAME=0.18.1-android.1
export BAELL_VERSION_CODE=180101
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
