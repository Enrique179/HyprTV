# Pruebas y clippy del núcleo Rust en el PC. Argumentos extra van a `cargo test` (filtro: .\scripts\test-rust.ps1 jokes).
$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot)
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\hyprtv-target"

cargo clippy -p hyprcore --all-targets
if ($LASTEXITCODE -ne 0) { throw "clippy con avisos" }
cargo test -p hyprcore @args
