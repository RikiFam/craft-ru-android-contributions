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
The prior suite launched on Galaxy Tab S7+ (Android 13). The 0.37-based update was
still being built and tested when this snapshot was recorded. Historical reports in
the subdirectories describe their own versions; they do not certify this whole snapshot.
This export has not undergone a fresh full Android/desktop build or complete device test.
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

## Download the complete source bundle

[Download the source ZIP](downloads/contribution-2026-10-29.zip) and extract it into a fresh directory, or clone this repository. ZIP SHA-256: `787eceeac4b42673fc047c461dc1e0447dd4d8cdffbfd41a35d34bfc248074b1`.
