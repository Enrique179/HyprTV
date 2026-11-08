# Compila el núcleo Rust para la TV (armeabi-v7a) y arm64, y deja las .so fuera de OneDrive, donde las lee Gradle.
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot
Set-Location $root

$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$env:ANDROID_NDK_HOME = "$env:LOCALAPPDATA\Android\Sdk\ndk\27.2.12479018"
$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\hyprtv-target"

cargo ndk -t armeabi-v7a -t arm64-v8a -o "$env:LOCALAPPDATA\hyprtv-jni" build --release -p hyprcore
if ($LASTEXITCODE -ne 0) { throw "cargo ndk falló" }
