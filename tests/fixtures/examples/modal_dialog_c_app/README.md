# modal_dialog_c_app

This is a standalone Win32 C modal dialog sample:
- It launches a modal dialog from dialog resources.
- It includes an edit field and an uppercase checkbox.
- Pressing OK shows a greeting message that uses the entered value.
- Cancel exits without showing a greeting.

This sample intentionally uses dialog APIs such as `DialogBoxParamW` and `EndDialog`.

## Build (Linux host with mingw-w64)

```bash
x86_64-w64-mingw32-windres -I. -o dialog.res.o dialog.rc
x86_64-w64-mingw32-gcc -municode -O1 -o modal_dialog_c_app.exe main.c dialog.res.o -luser32
```

## Run (Windows)

```powershell
.\modal_dialog_c_app.exe
```
