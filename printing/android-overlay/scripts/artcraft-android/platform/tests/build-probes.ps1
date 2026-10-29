param(
    [string]$Sdk='E:/Codex/androd lin/toolchain/android-sdk',
    [string]$Jdk='E:/Codex/androd lin/toolchain/jdk-17'
)
$ErrorActionPreference='Stop'
$root=(Resolve-Path "$PSScriptRoot/../../../..").Path
$output=Join-Path $root 'tmp/artcraft/android/direct-print-probes'
New-Item -ItemType Directory -Force "$output/classes","$output/dex" | Out-Null
$sources=@('CraftIpp.java','CraftPrintSettings.java','CraftPwg.java','CraftMaintenance.java') | ForEach-Object {Join-Path "$PSScriptRoot/../java/ai/storyteller/android" $_}
$sources+=@("$PSScriptRoot/PrinterProbe.java","$PSScriptRoot/PrintLogicTest.java")
& "$Jdk/bin/javac.exe" -encoding UTF-8 -source 8 -target 8 -classpath "$Sdk/platforms/android-35/android.jar" -d "$output/classes" @sources
if($LASTEXITCODE -ne 0){throw 'Probe compilation failed'}
& "$Jdk/bin/java.exe" -cp "$output/classes;$Sdk/platforms/android-35/android.jar" ai.storyteller.android.PrintLogicTest
if($LASTEXITCODE -ne 0){throw 'Print logic tests failed'}
$env:JAVA_HOME=$Jdk
$classes=@(Get-ChildItem "$output/classes" -Recurse -Filter '*.class' | ForEach-Object FullName)
& "$Sdk/build-tools/35.0.0/d8.bat" --lib "$Sdk/platforms/android-35/android.jar" --min-api 29 --output "$output/dex" @classes
if($LASTEXITCODE -ne 0){throw 'Probe DEX failed'}
& "$Jdk/bin/jar.exe" cf "$output/probes.jar" -C "$output/dex" classes.dex
if($LASTEXITCODE -ne 0){throw 'Probe JAR failed'}
Write-Output "$output/probes.jar"
