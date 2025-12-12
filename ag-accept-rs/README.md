# Ag-Accept (Rust Version)

This is a lightweight Rust rewrite of the Ag-Accept automation tool.

## Prerequisites
- Rust (Cargo) installed.

## Build
```powershell
cd ag-accept-rs
cargo build --release
```
The executable will be at `target/release/ag-accept-rs.exe`.

## Run
```powershell
./target/release/ag-accept-rs.exe
```
Or during development:
```powershell
cargo run
```

## Configuration
The app looks for `config.json` in:
1. The standard configuration directory (e.g., `AppData/Local/RyosukeMondo/ag-accept/config.json`).
2. The current working directory.

If not found, it generates a default configuration in the current directory.

## Structure
- `src/main.rs`: Entry point.
- `src/config.rs`: Configuration loading.
- `src/automation.rs`: Main automation loop.
- `src/services/window.rs`: Window finding logic (using UI Automation TreeWalker).
- `src/services/query.rs`: Recursive text search logic.
