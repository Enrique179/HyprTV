# Compila (Rust + APK release) e instala en la TV conectada por ADB. `-SinRust` salta el núcleo si no cambió.
param([switch]$SinRust)
$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot
Set-Location $root

$env:JAVA_HOME = "C:\Program Files\Android\Android Studio\jbr"
$env:Path = "$env:JAVA_HOME\bin;$env:USERPROFILE\.cargo\bin;$env:Path"
$adb = "$env:LOCALAPPDATA\Android\Sdk\platform-tools\adb.exe"

if (-not $SinRust) { & "$PSScriptRoot\build-rust.ps1"; if ($LASTEXITCODE -ne 0) { throw "falló el núcleo Rust" } }
.\gradlew.bat :app:assembleRelease -q
if ($LASTEXITCODE -ne 0) { throw "falló Gradle" }
& $adb install -r "$env:LOCALAPPDATA\hyprtv-build\app\outputs\apk\release\app-release.apk"
