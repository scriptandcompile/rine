# all_controls_c_app

This is a standalone Win32 C sample intended for UI-fixture testing.
It creates a single top-level window that contains a broad set of standard Win32 and common controls in one place.

Included controls:
- Static labels and group boxes
- Buttons: push button, default push button, checkbox, radio buttons
- Edit controls: single-line, password, multiline, buddy edit for up-down
- List controls: list box, combo box, list view, tree view, tab control, header control
- Range/progress controls: progress bar, trackbar, up-down
- Date/time controls: date-time picker, month calendar
- Misc common controls: hotkey control, horizontal and vertical scroll bars, status bar
- Native top-level menu

## Build (Linux host with mingw-w64)

```bash
x86_64-w64-mingw32-gcc -municode -O1 -o all_controls_c_app.exe main.c -lgdi32 -lcomctl32
```

Or build all fixture examples from repo root:

```bash
./scripts/build-fixture-examples.sh
```

## Run (Windows)

```powershell
.\all_controls_c_app.exe
```
