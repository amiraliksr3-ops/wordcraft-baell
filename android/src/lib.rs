//! WordCraft for Android — the Baell port's host crate.
//!
//! WordCraft's front end (`wordcraft-ui-egui`, the Word-style ribbon UI) is swappable and knows
//! nothing about platforms: a host builds [`WordApp`] with [`Services`] and drives it through
//! eframe. This crate is the Android host, the same shape as the desktop app
//! (`apps/wordcraft/src/main.rs`) minus what a phone doesn't have: no native window dialogs
//! (the app's built-in file browser opens instead, which works without any system dialog), no
//! printers, no control server, no self-updates.
//!
//! The activity is a plain `android.app.NativeActivity` (see `app/src/main/AndroidManifest.xml`):
//! the system loads `libwordcraft_android.so`, finds the exported `android_main` below, and the
//! crate builds an external winit event loop so eframe can run on it
//! (`eframe::create_native` + `EventLoopBuilderExtAndroid::with_android_app`).
//!
//! Everything the user keeps lives under the activity's internal data directory:
//! `ui.json` (preferences), `recovery/` (document recovery), `fonts/`, `Templates/` and
//! `custom-dictionary.txt`. Files the app "downloads" (exports without a dialog path) are
//! written next to them, and can be reached from other apps through the device's file manager.
//!
//! ## Unsafe exception
//!
//! The workspace forbids unsafe code. This crate declares one documented exception in its
//! `Cargo.toml`: `android_main` must carry `#[unsafe(no_mangle)]` so the NativeActivity glue can
//! find it by symbol name. Nothing else in the crate is unsafe.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

// The crate only does anything on Android; on other targets it is an empty library so the
// workspace still checks and formats it (`cargo fmt --all`, `cargo clippy --workspace`).
#[cfg(target_os = "android")]
mod host;

#[cfg(target_os = "android")]
pub use host::run;
