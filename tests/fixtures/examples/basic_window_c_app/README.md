# basic_window_c_app

This is a standalone Win32 C sample with small real behavior:
- It creates a top-level window.
- It adds a button and a status text control.
- Clicking the button increments a counter and updates the status text.
- Every 5 clicks, it updates the window title.
- Pressing `Esc` closes the app.

## Why this is outside fixture builds

Integration fixture builds only compile `*.c` files under `tests/fixtures/src`.
This sample lives in `tests/fixtures/examples/basic_window_c_app`, so it is not included.

Source of that behavior: `scripts/build-integration-prereqs.sh`
- `SRC_DIR="$REPO_ROOT/tests/fixtures/src"`
- `find "$SRC_DIR" -type f -name '*.c'`

## Build (Linux host with mingw-w64)

```bash
x86_64-w64-mingw32-gcc -municode -O1 -o basic_window_c_app.exe main.c -lgdi32 -lcomctl32
```

Or build all fixture examples from repo root:

```bash
./scripts/build-fixture-examples.sh
```

## Run (Windows)

```powershell
.\basic_window_c_app.exe
```
