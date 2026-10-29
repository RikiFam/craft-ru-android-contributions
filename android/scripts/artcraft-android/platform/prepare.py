"""Wire the Android-only SDK boundary without changing desktop binaries."""
from pathlib import Path
import os
root = Path(__file__).resolve().parents[3]
base = root / 'scripts/artcraft-android'
platform = base / 'platform'
projects = [base, *[p for p in (base/'suite').iterdir() if (p/'Cargo.toml').exists() and p.name != 'launcher']]
for project in projects:
    manifest = project/'Cargo.toml'
    text = manifest.read_text('utf-8')
    if 'artcraft-android-platform =' not in text:
        dependency = os.path.relpath(platform, project).replace('\\','/')
        text = text.replace('[dependencies]', '[dependencies]\nartcraft-android-platform = { path = "'+dependency+'" }')
        manifest.write_text(text,'utf-8')
    xml = project/'AndroidManifest.xml'
    text = xml.read_text('utf-8').replace('android:hasCode="false"','android:hasCode="true"').replace('android.app.NativeActivity','ai.storyteller.android.CraftActivity')
    text = text.replace('versionCode="3"','versionCode="4"').replace('-preview3"','-preview4"')
    xml.write_text(text,'utf-8')
    source = project/'src/lib.rs'
    text = source.read_text('utf-8')
    marker = 'fn android_main(android:'
    if 'artcraft_android_platform::initialize' not in text:
        start = text.index('{',text.index(marker))+1
        text = text[:start]+'''
    // SAFETY: AndroidApp supplies live JVM/activity handles for this process.
    if let Err(error) = unsafe { artcraft_android_platform::initialize(android.vm_as_ptr().cast(), android.activity_as_ptr().cast()) } {
        log::error!("{error}");
        return;
    }
'''+text[start:]
        source.write_text(text,'utf-8')
for filename in ('build.ps1','build-suite.ps1'):
    path=base/filename
    text=path.read_text('utf-8')
    if 'platform/build-dex.ps1' not in text:
        insert='''    $platformDex = & "$PSScriptRoot/platform/build-dex.ps1" -Sdk $Sdk -Jdk $Jdk -Output $LinkAlias
'''
        text=text.replace('    Add-Type -AssemblyName System.IO.Compression.FileSystem',insert+'    Add-Type -AssemblyName System.IO.Compression.FileSystem')
        text=text.replace('    try {\n        [IO.Compression.ZipFileExtensions]', '    try {\n        [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $platformDex, "classes.dex", [IO.Compression.CompressionLevel]::Optimal) | Out-Null\n        [IO.Compression.ZipFileExtensions]')
        # Dependency lockfile is updated by Cargo for the new local platform crate.
        text=text.replace('cargo rustc --locked','cargo rustc')
        path.write_text(text,'utf-8')

# WordCraft writes through one atomic-save boundary; propagate provider failures
# before the engine marks the document saved. Android-only dependency.
engine=root/'tmp/artcraft/wordcraft/repo/crates/engine'
toml=engine/'Cargo.toml';text=toml.read_text('utf-8')
if 'artcraft-android-platform' not in text:
    text+='\n[target.\'cfg(target_os = "android")\'.dependencies]\nartcraft-android-platform = { path = "'+os.path.relpath(platform,engine).replace('\\','/')+'" }\n'
    toml.write_text(text,'utf-8')
source=engine/'src/io/safe_save.rs';text=source.read_text('utf-8')
if 'artcraft_android_platform::commit' not in text:
    text=text.replace('    written.map_err(explain)','''    written.map_err(explain)?;
    #[cfg(target_os = "android")]
    artcraft_android_platform::commit(path).map_err(io::Error::other)?;
    Ok(())''',1)
    source.write_text(text,'utf-8')
