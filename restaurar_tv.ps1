# Devuelve la TV al launcher de Google TV y apaga HyprTV. Necesita la TV conectada por ADB inalámbrico.
#   .\restaurar_tv.ps1                 -> inicio de Google TV + apaga el servicio (HyprTV queda instalado)
#   .\restaurar_tv.ps1 -Desinstalar    -> además desinstala HyprTV
param([switch]$Desinstalar)

$adb = "$env:LOCALAPPDATA\Android\Sdk\platform-tools\adb.exe"
$servicio = "com.hyprtv/com.hyprtv.ScreenService"

& $adb devices
Write-Host "1) Inicio de Google TV"
& $adb shell cmd package set-home-activity com.google.android.apps.tv.launcherx/.home.HomeActivity

Write-Host "2) Servicio de accesibilidad fuera (sin tocar los demás)"
$actuales = (& $adb shell settings get secure enabled_accessibility_services).Trim()
if ($actuales -and $actuales -ne "null") {
    $resto = ($actuales -split ":" | Where-Object { $_ -and $_ -ne $servicio }) -join ":"
    if ($resto) { & $adb shell settings put secure enabled_accessibility_services $resto }
    else { & $adb shell settings delete secure enabled_accessibility_services; & $adb shell settings put secure accessibility_enabled 0 }
}

if ($Desinstalar) {
    Write-Host "3) Desinstalando HyprTV"
    & $adb uninstall com.hyprtv
}
& $adb shell input keyevent KEYCODE_HOME
Write-Host "Listo."
