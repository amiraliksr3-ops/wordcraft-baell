# WordCraft (Baell) — the Android port's interface

What the Android build runs and what is Android-specific. Be honest about status here: this
port is young, and this file is updated with it.

## What runs on Android

The Android app runs **the real WordCraft front end** (`wordcraft-ui-egui`), the same one as
the desktop apps: the ribbon, styles, tables, track changes, references, find and replace —
the full command set, driven by the same engine. Nothing was reimplemented or faked: the
desktop UI is drawn with egui, which routes touch input through the same pointer events a
mouse would produce, so the interface is usable by touch out of the box (tap to select and
press buttons, drag to scroll, long-press for context menus, pinch and two-finger gestures
where egui reports them). Text input goes through the Android soft keyboard via eframe's IME
support.

Because the whole front end is reused, the Android app follows WordCraft's own UI
conventions — including File ▸ Options, the About window with contributors, and the command
search — rather than a separate mobile shell.

## Android-specific behaviour

- **Files**: there is no native file dialog on Android, so File ▸ Open / Save As / Export /
  Insert Picture / recipient lists open WordCraft's own built-in file browser (#576) instead.
  It lists folders and files, navigates and picks paths, and is the same browser the Linux
  desktop uses when no system dialog is available. It reads paths the app may access: the
  app's own folders always, and shared storage on Android 12L and below (the app requests
  `READ_EXTERNAL_STORAGE` only up to API 32).
- **Downloads**: exports the app would hand to a "download" go into the app's external files
  directory (`Android/data/ai.baell.word/files/Downloads` on the device). On Android 10 and
  older any file manager can open it; on Android 11 and newer the system restricts that folder,
  so use File ▸ Save As (the built-in browser) when the file has to go somewhere else.
- **Everything the user keeps** lives in the app's internal data directory: preferences
  (`ui.json`), document recovery (`recovery/`), user-added fonts (`fonts/`), personal
  templates (`Templates/`) and the custom dictionary. The Android recovery pass is wired the
  same way as on the desktop.
- **Not on Android** (desktop-only plumbing): printers and native printing, the localhost
  control server and MCP bridge, self-updates, window-geometry saving (the system owns the
  window).

## Known limitations / future work

- The interface is the desktop layout at phone sizes: usable (egui touch input, soft
  keyboard), but a touch-adapted compact ribbon/navigation bar is future work — see the
  roadmap. Command search (Ctrl-style) is the practical shortcut for reaching a command
  quickly on a small screen.
- Opening documents straight from other apps ("Open with") needs an SAF/JNI bridge, which
  the NativeActivity host doesn't have yet; open files through the built-in browser for now.
- Interface languages other than English need the `CRAFT_I18N_DIR` build input (the release
  workflow builds without it for now); the interface falls back to English.
