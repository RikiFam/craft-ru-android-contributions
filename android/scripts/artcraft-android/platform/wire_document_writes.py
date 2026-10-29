"""Publish document engine atomic writes back to Android document providers."""
from wire_media import dependency, edit

engine = dependency('deckcraft')
edit(engine/'src/io.rs',
     '        return Err(err(e));\n    }\n    Ok(())',
     '        return Err(err(e));\n    }\n    #[cfg(target_os = "android")]\n    artcraft_android_platform::commit(&target_path)?;\n    Ok(())')

engine = dependency('designcraft')
edit(engine/'src/atomic.rs',
     '    craft_atomic::atomic_write(path, bytes)',
     '''    craft_atomic::atomic_write(path, bytes)?;
    #[cfg(target_os = "android")]
    artcraft_android_platform::commit(path).map_err(io::Error::other)?;
    Ok(())''')
edit(engine/'src/atomic.rs',
     '    })\n}',
     '''    })?;
    #[cfg(target_os = "android")]
    artcraft_android_platform::commit(path).map_err(io::Error::other)?;
    Ok(())
}''')
