param([string]$Sdk,[string]$Jdk,[string]$Output)
$ErrorActionPreference='Stop'
$classes=Join-Path $Output 'platform-classes'
$dex=Join-Path $Output 'platform-dex'
New-Item -ItemType Directory -Force -Path $classes,$dex | Out-Null
$sources=@(Get-ChildItem (Join-Path $PSScriptRoot 'java') -Recurse -Filter '*.java' | ForEach-Object FullName)
& "$Jdk/bin/javac.exe" -encoding UTF-8 -source 8 -target 8 -classpath "$Sdk/platforms/android-35/android.jar" -d $classes @sources
if($LASTEXITCODE -ne 0){throw 'Android platform compilation failed'}
$compiled=@(Get-ChildItem $classes -Recurse -Filter '*.class' | ForEach-Object FullName)
$env:JAVA_HOME=$Jdk
& "$Sdk/build-tools/35.0.0/d8.bat" --lib "$Sdk/platforms/android-35/android.jar" --min-api 29 --output $dex @compiled
if($LASTEXITCODE -ne 0){throw 'Android DEX compilation failed'}
Join-Path $dex 'classes.dex'
