param(
    [switch]$PackageOnly,
    [string]$Sdk = 'E:\Codex\androd lin\toolchain\android-sdk',
    [string]$Ndk = 'E:\Codex\androd lin\android-ndk-r27d',
    [string]$Jdk = 'E:\Codex\androd lin\toolchain\jdk-17',
    [string]$LinkAlias = 'E:\Codex\ArtCraftAndroidLink'
)
$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$output = Join-Path $root 'tmp\artcraft\android'
New-Item -ItemType Directory -Path $output -Force | Out-Null
if ($LinkAlias -match '[^\x00-\x7f]') { throw 'LinkAlias must contain only ASCII characters' }
if (Test-Path -LiteralPath $LinkAlias) {
    $existing = Get-Item -LiteralPath $LinkAlias
    if ($existing.LinkType -ne 'Junction' -or [IO.Path]::GetFullPath(@($existing.Target)[0]) -ne $output) {
        throw 'LinkAlias already exists and does not point to this build directory'
    }
} else {
    New-Item -ItemType Junction -Path $LinkAlias -Target $output | Out-Null
}
$env:JAVA_HOME = $Jdk
$env:PATH = "$Jdk\bin;$env:PATH"
$llvm = Join-Path $Ndk 'toolchains\llvm\prebuilt\windows-x86_64\bin'
$env:CC_aarch64_linux_android = Join-Path $llvm 'aarch64-linux-android29-clang.cmd'
$env:AR_aarch64_linux_android = Join-Path $llvm 'llvm-ar.exe'
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER = $env:CC_aarch64_linux_android
$env:CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS = '-C link-arg=-Wl,-z,max-page-size=16384'
$env:CARGO_TARGET_DIR = Join-Path $output 'target'
$env:CARGO_BUILD_JOBS = '4'
$env:CRAFT_FONTS_DIR = Join-Path $root 'tmp\artcraft\craft-fonts'
$env:CRAFT_I18N_DIR = Join-Path $root 'tmp\artcraft\craft-i18n'
$env:ARTCRAFT_LINK_FROM = $output
$env:ARTCRAFT_LINK_TO = $LinkAlias
$env:ARTCRAFT_NDK_CLANG = Join-Path $llvm 'clang.exe'
Push-Location $PSScriptRoot
try {
    python "$PSScriptRoot\apply_source_patch.py"
    if ($LASTEXITCODE -ne 0) { throw 'Source compatibility patch failed' }
    $linker = Join-Path $output 'android-link.exe'
    if (!(Test-Path -LiteralPath $linker) -or (Get-Item "$PSScriptRoot\android-link.rs").LastWriteTimeUtc -gt (Get-Item $linker).LastWriteTimeUtc) {
        rustc --edition 2024 "$PSScriptRoot\android-link.rs" -o $linker
        if ($LASTEXITCODE -ne 0) { throw 'Linker adapter build failed' }
    }
    # NDK tools narrow Unicode arguments to the system codepage. Use an ASCII
    # junction for link inputs only, preserving the compiled Cargo cache in place.
    if (-not $PackageOnly) {
        cargo rustc --locked --release --target aarch64-linux-android --lib -- -C "linker=$linker" -C 'link-arg=--target=aarch64-linux-android29'
        if ($LASTEXITCODE -ne 0) { throw 'Android compilation failed' }
    }
    $library = Join-Path $env:CARGO_TARGET_DIR 'aarch64-linux-android\release\libphotocraft_android_ru.so'
    $symbols = & "$llvm\llvm-readelf.exe" --wide --dyn-syms $library
    if ($LASTEXITCODE -ne 0 -or -not ($symbols -match 'ANativeActivity_onCreate') -or -not ($symbols -match 'android_main')) {
        throw 'Android entry points are missing'
    }
    $dynamic = & "$llvm\llvm-readelf.exe" --dynamic $library
    if ($LASTEXITCODE -ne 0) { throw 'Could not inspect Android library dependencies' }
    $tools = Join-Path $Sdk 'build-tools\35.0.0'
    $unsigned = Join-Path $output 'photocraft-unsigned.apk'
    $resources = Join-Path $output 'photocraft-res'
    New-Item -ItemType Directory -Path "$resources\drawable-nodpi" -Force | Out-Null
    Copy-Item -LiteralPath "$root\tmp\artcraft\photocraft\repo\assets\app-icon\hicolor\512x512\apps\ai.storyteller.photocraft.png" -Destination "$resources\drawable-nodpi\app_icon.png"
    & "$tools\aapt.exe" package -f -M "$PSScriptRoot\AndroidManifest.xml" -S $resources -I "$Sdk\platforms\android-35\android.jar" -F $unsigned
    if ($LASTEXITCODE -ne 0) { throw 'APK manifest packaging failed' }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    Add-Type -AssemblyName System.IO.Compression
    $zip = [IO.Compression.ZipFile]::Open($unsigned, [IO.Compression.ZipArchiveMode]::Update)
    try {
        [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $library, 'lib/arm64-v8a/libphotocraft_android_ru.so', [IO.Compression.CompressionLevel]::Optimal) | Out-Null
        if ($dynamic -match 'libc\+\+_shared.so') {
            $cpp = Join-Path $Ndk 'toolchains\llvm\prebuilt\windows-x86_64\sysroot\usr\lib\aarch64-linux-android\libc++_shared.so'
            [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $cpp, 'lib/arm64-v8a/libc++_shared.so', [IO.Compression.CompressionLevel]::NoCompression) | Out-Null
        }
    } finally { $zip.Dispose() }
    $aligned = Join-Path $output 'photocraft-aligned.apk'
    & "$tools\zipalign.exe" -P 16 -f 4 $unsigned $aligned
    if ($LASTEXITCODE -ne 0) { throw 'APK alignment failed' }
    # Local preview key, never committed or used for publishing. Preserve it for future updates.
    $key = Join-Path $output 'preview.keystore'
    if (-not (Test-Path -LiteralPath $key)) {
        & "$Jdk\bin\keytool.exe" -genkeypair -keystore $key -storepass android -keypass android -alias androiddebugkey -dname 'CN=Local PhotoCraft Preview' -keyalg RSA -keysize 2048 -validity 10000
        if ($LASTEXITCODE -ne 0) { throw 'Local signing key creation failed' }
    }
    $apk = Join-Path $output 'PhotoCraft-RU-TabS7Plus-preview.apk'
    & "$tools\apksigner.bat" sign --ks $key --ks-pass pass:android --out $apk $aligned
    if ($LASTEXITCODE -ne 0) { throw 'APK signing failed' }
    & "$tools\apksigner.bat" verify --verbose $apk
    if ($LASTEXITCODE -ne 0) { throw 'APK signature verification failed' }
    & "$tools\aapt.exe" dump badging (Join-Path $LinkAlias 'PhotoCraft-RU-TabS7Plus-preview.apk')
    if ($LASTEXITCODE -ne 0) { throw 'APK manifest inspection failed' }
    & "$tools\zipalign.exe" -c -P 16 4 (Join-Path $LinkAlias 'PhotoCraft-RU-TabS7Plus-preview.apk')
    if ($LASTEXITCODE -ne 0) { throw 'Final APK alignment verification failed' }
    Get-FileHash -LiteralPath $apk -Algorithm SHA256
} finally { Pop-Location }
