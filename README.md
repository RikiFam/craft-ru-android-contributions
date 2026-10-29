# Russian translations and experimental Android ports

Contribution material for the original [storytold projects](https://github.com/storytold).
This is an unofficial source snapshot, not an official release or a fully tested update.
It includes Russian translation changes for 13 editors and their launcher, plus native
Rust/egui Android ARM64 hosts. Original copyright, licenses and attribution are retained.
The exact upstream commits and patch hashes are in `sources.json`.

## Reproduce the source trees

Use a fresh directory on Windows with Python 3 and Git:

```powershell
python prepare_sources.py
rustup target add aarch64-linux-android
```

Install Rust, JDK 17, Android SDK platform 35 / Build Tools 35.0.0 and NDK r27d.
Pass your own tool paths; the historical defaults in the build scripts are author-local.
The ASCII junction path must be unused and on a suitable local drive:

```powershell
./scripts/artcraft-android/build.ps1 -Sdk C:/Android/Sdk -Ndk C:/Android/ndk-r27d -Jdk C:/Java/jdk-17 -LinkAlias C:/CraftAndroidBuild
./scripts/artcraft-android/build-suite.ps1 -App wordcraft -Sdk C:/Android/Sdk -Ndk C:/Android/ndk-r27d -Jdk C:/Java/jdk-17 -LinkAlias C:/CraftAndroidBuild
```

Build PhotoCraft first: it initializes the shared development signing material.
PdfCraft additionally requires rfd 0.17.2 in the Cargo registry before its preparation step.
The scripts create local preview APKs; development signing is not production signing.
No signing keys, installed programs, private documents or device logs are included.

## Validation and limits

All 14 application patches were checked against their recorded upstream Git trees.
The source chat records installation, device version and Russian window checks for all
14 updated Android packages on Galaxy Tab S7+ (Android 13). PhotoCraft drawing/PSD saving
and opening a test PDF in PdfCraft were checked. These are launch and selected workflow
checks, not complete functional coverage. Shared craft-i18n: 26 tests and Clippy passed.
Windows update work was still in progress at this snapshot. The export itself has not
undergone an independent full rebuild. Historical subdirectory reports describe their
own versions; the dated update report describes these newer device checks.
Android document pickers use app folders; SAF, sharing, rotation and every editor function
are not comprehensively tested. This is contribution material for review.

## Brand and distribution

ArtCraft Launcher brand assets are separately restricted. A fork maintained solely to
propose changes upstream may retain the original marks. A redistributed modified launcher
must remove/replace the marks and be renamed, unless the authors give permission.
See the original `docs/brand/LICENSE-brand.txt`. Do not publish the locally signed launcher
APK as an official release. Other projects retain their own asset licenses.

MIT OR Apache-2.0 for contributions and applicable upstream code; third-party assets
retain their original terms. See LICENSE files and the source repositories' attribution.

## Download this source snapshot

[Source ZIP](downloads/contribution-a209e3b.zip), SHA-256: `a06ab614753258a1fa4235d92a39a643f6414b504e3c0c001774e120ba5b6f1d`. The archive contains source files, not APKs.
