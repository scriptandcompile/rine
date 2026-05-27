# Fixture Examples

These examples are separate from integration fixture sources.

- Integration fixtures are built only from `tests/fixtures/src`.
- Examples in `tests/fixtures/examples` are not included in fixture build scripts.

## Included examples

- `basic_window_c_app`
- `modal_dialog_c_app`

## Build both examples

From the repository root:

```bash
./scripts/build-fixture-examples.sh
```

Output binaries:

- `tests/fixtures/examples/bin/basic_window_c_app.exe`
- `tests/fixtures/examples/bin/modal_dialog_c_app.exe`
