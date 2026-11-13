# WordCraft (Baell) — the Android port's interface

What the Android build runs and what is Android-specific. Be honest about status here: this
port is young, and this file is updated with it.

## What runs on Android

The Android app runs **the real WordCraft front end** (`wordcraft-ui-egui`), the same engine and
command set as the desktop apps: styles, tables, track changes, references, find and replace,
proofing and the full ribbon. Nothing was reimplemented or faked.

The front end is drawn with egui, which reports touch as pointer input, so the document is
edited by tapping and by the Android soft keyboard (through eframe's IME support). Two-finger
scrolling and pinch zoom work where egui reports the gestures.

## The phone layout

On Android, a window narrower than **720 points** gets a touch shell instead of the desktop
chrome. The shell is the phone's: a desktop window keeps its mouse-and-keyboard chrome even when
it is narrow, so the desktop layout is unchanged at every size. On a tablet in landscape (720
points or wider) the desktop chrome returns; so does the desktop shell if the window is widened.

- **Top bar (52 px)**: File, Save, Undo, Redo and Search, each a 52 × 48 touch target. File opens
  the Backstage; Search opens command search.
- **The page**, full width. The title bar, tab strip, status bar, rulers and task panes are not
  drawn: together they used most of a phone screen.
- **Bottom tab bar (58 px)**: the ribbon tabs as pills. Tapping one switches to that tab and
  shows its controls in a sheet above the bar — the same groups the desktop ribbon draws
  (`ribbon::tab_content`), so every command has the same button, menu and tooltip.
- **Ribbon sheet**: up to 42 % of the screen height, scrolling sideways like the ribbon. The
  `▾` button hides it; tapping any tab brings it back.
- **Touch metrics**: ribbon buttons are wider and taller than on the desktop (big buttons at
  least 56 px wide, small ones 34 × 32), and general widgets (check boxes, list rows, dialog
  buttons) get a 34 px minimum hit area while the shell is on. The selection mini toolbar keeps
  the desktop's control sizes: its fixed two-row width was measured for them, and finger-sized
  buttons would spill out of it and off a narrow screen.
- **Dialogs** clamp their width to the screen (the built-in file browser, the first one that
  mattered, is 620 px on the desktop and the window width minus 24 px on a phone).
- **Backstage**: the left navigation becomes a 72 px icon rail.
- **File ▸ Options**: the pane list becomes a scrolling row above the settings instead of a
  170 px rail beside them.
- **Status messages** (Saved, couldn't write, …) appear as a short-lived toast above the tab bar,
  because there is no status bar.

## Android-specific behaviour

- **Files**: there is no native file dialog on Android, so File ▸ Open / Save As / Export /
  Insert Picture / recipient lists open WordCraft's own built-in file browser (#576) instead.
  It starts in the app's `Documents` folder, where the app can always write, and reads paths the
  app may access (the app's own folders always, and shared storage on Android 12L and below —
  the app requests `READ_EXTERNAL_STORAGE` only up to API 32).
- **Saving**: Save and Save As write to the path the built-in browser picked, and that file
  becomes the document's file, so the title, the next Save and AutoSave use it from then on —
  a document opened from a folder is saved back to that folder, not to a copy elsewhere. A save
  or export that only has a file name (an untitled document, a PDF export) goes to the app's
  `Documents` folder (`Android/data/ai.baell.word/files/Documents` on the device); for a save,
  that folder then becomes the document's folder.
- **Scripts**: Persian, Arabic, Hebrew and CJK text is shaped and laid out by the same engine code
  as the desktop: the full Unicode bidirectional algorithm, Arabic joining and the complex-script
  font properties. The glyphs come from the device's own fonts (`/system/fonts`, plus
  `/product/fonts` and `/vendor/fonts` on newer releases), because this build ships no bundled
  Arabic face; on Android those include Noto Naskh Arabic and the CJK faces. Typing Persian as the
  first letters of an empty paragraph makes the paragraph right-to-left, as on the desktop, and the
  Font list shows the device's installed families as well as the bundled ones. The system font
  catalog is scanned on a background thread at startup, so the first Persian character laid out
  does not hold up the frame.
- **Everything the user keeps** lives in the app's internal data directory: preferences
  (`ui.json`), document recovery (`recovery/`), user-added fonts (`fonts/`), personal
  templates (`Templates/`) and the custom dictionary. The Android recovery pass is wired the
  same way as on the desktop.
- **Not on Android** (desktop-only plumbing): printers and native printing, the localhost
  control server and MCP bridge, self-updates, window-geometry saving (the system owns the
  window).

## Known limitations / future work

- **Where files live**: the release build targets Android 15, so scoped storage applies and the
  app writes inside its own external folder (`Android/data/ai.baell.word/files`). A file manager
  can reach it on many devices, but not all. A system "save to Documents/Downloads" flow
  (SAF/JNI, `MediaStore`) is future work; until then the built-in browser's folder field and
  Save As keep exports inside the app's own folder.
- Opening documents straight from other apps ("Open with") needs the same SAF/JNI bridge, which
  the NativeActivity host doesn't have yet; open files through the built-in browser for now.
- A hardware keyboard is not required, but some desktop shortcuts (Alt chords, function keys)
  have no on-screen equivalent; Search and the ribbon are the ways to reach those commands.
- Interface languages other than English need the `CRAFT_I18N_DIR` build input (the release
  workflow builds without it for now); the interface falls back to English.
