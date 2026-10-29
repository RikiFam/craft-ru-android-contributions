param([string]$Sdk='E:/Codex/androd lin/toolchain/android-sdk',[string]$Jdk='E:/Codex/androd lin/toolchain/jdk-17')
$ErrorActionPreference='Stop'
$root=(Resolve-Path "$PSScriptRoot/../../../..").Path
$output=Join-Path $root 'tmp/artcraft/android/direct-print-test-apk'
New-Item -ItemType Directory -Force "$output/classes","$output/dex" | Out-Null
$sources=@(Get-ChildItem "$PSScriptRoot/../java" -Recurse -Filter '*.java' | ForEach-Object FullName)
$sources+="$PSScriptRoot/PrintProbeActivity.java"
& "$Jdk/bin/javac.exe" -encoding UTF-8 -source 8 -target 8 -classpath "$Sdk/platforms/android-35/android.jar" -d "$output/classes" @sources
if($LASTEXITCODE -ne 0){throw 'Test APK compilation failed'}
$env:JAVA_HOME=$Jdk
$classes=@(Get-ChildItem "$output/classes" -Recurse -Filter '*.class' | ForEach-Object FullName)
& "$Sdk/build-tools/35.0.0/d8.bat" --lib "$Sdk/platforms/android-35/android.jar" --min-api 29 --output "$output/dex" @classes
if($LASTEXITCODE -ne 0){throw 'Test APK DEX failed'}
& "$Sdk/build-tools/35.0.0/aapt.exe" package -f -M "$PSScriptRoot/AndroidManifest.xml" -I "$Sdk/platforms/android-35/android.jar" -F "$output/unsigned.apk"
if($LASTEXITCODE -ne 0){throw 'Test APK manifest failed'}
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip=[IO.Compression.ZipFile]::Open("$output/unsigned.apk",[IO.Compression.ZipArchiveMode]::Update)
try{[IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip,"$output/dex/classes.dex",'classes.dex') | Out-Null}finally{$zip.Dispose()}
& "$Sdk/build-tools/35.0.0/zipalign.exe" -f 4 "$output/unsigned.apk" "$output/test.apk"
if($LASTEXITCODE -ne 0){throw 'Test APK alignment failed'}
& "$Sdk/build-tools/35.0.0/apksigner.bat" sign --ks "$root/tmp/artcraft/android/preview.keystore" --ks-pass pass:android --key-pass pass:android "$output/test.apk"
if($LASTEXITCODE -ne 0){throw 'Test APK signing failed'}
Write-Output "$output/test.apk"
