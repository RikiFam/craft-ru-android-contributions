param([Parameter(Mandatory=$true)][string]$Sdk,[Parameter(Mandatory=$true)][string]$Jdk,[string]$Adb,[string]$Serial,[switch]$RunOnDevice)
$ErrorActionPreference='Stop'
$root=(Resolve-Path "$PSScriptRoot/../../../..").Path
$o=Join-Path $root 'tmp/artcraft/android/touchpad-sdk-test'
New-Item -ItemType Directory -Force "$o/classes","$o/dex" | Out-Null
& "$jdk/bin/javac.exe" -encoding UTF-8 -source 8 -target 8 -classpath "$sdk/platforms/android-35/android.jar" -d "$o/classes" "$PSScriptRoot/../java/ai/storyteller/android/CraftTouchpad.java" "$PSScriptRoot/TouchpadSdkTest.java"
if($LASTEXITCODE -ne 0){throw 'Java compilation failed'}
$env:JAVA_HOME=$jdk
$classes=@(Get-ChildItem "$o/classes" -Recurse -Filter '*.class' | ForEach-Object FullName)
& "$sdk/build-tools/35.0.0/d8.bat" --lib "$sdk/platforms/android-35/android.jar" --min-api 29 --output "$o/dex" @classes
if($LASTEXITCODE -ne 0){throw 'DEX compilation failed'}
& "$jdk/bin/jar.exe" cf "$o/test.jar" -C "$o/dex" classes.dex
if($LASTEXITCODE -ne 0){throw 'JAR packaging failed'}
if (-not $RunOnDevice) { Write-Output 'Compiled only; no device contacted'; return }
if (-not $Adb -or -not $Serial) { throw 'Provide Adb and Serial explicitly for device tests' }
& $Adb -s $Serial push "$o/test.jar" /data/local/tmp/artcraft-touchpad-sdk-test.jar
if($LASTEXITCODE -ne 0){throw 'ADB push failed'}
& $Adb -s $Serial shell 'CLASSPATH=/data/local/tmp/artcraft-touchpad-sdk-test.jar app_process / ai.storyteller.android.TouchpadSdkTest'
if($LASTEXITCODE -ne 0){throw 'Android SDK gesture tests failed'}
