"""Reproducible Android-only fixes on the pinned winit revision; cache stays untouched."""
from pathlib import Path
import shutil
root=Path(__file__).resolve().parents[3]
source=root/'tmp/artcraft/android/winit-source'
dest=root/'tmp/artcraft/android/vendor/winit'
if not dest.exists():
    shutil.copytree(source,dest,ignore=shutil.ignore_patterns('.git','target'))
p=dest/'src/platform_impl/android/mod.rs'
s=(source/'src/platform_impl/android/mod.rs').read_text('utf-8')
old='''                let phase = match motion_event.action() {'''
new='''                // Android touchpads report two-finger scrolling as ACTION_SCROLL, not Touch.
                if motion_event.action() == MotionAction::Scroll {
                    let pointer = motion_event.pointer_at_index(0);
                    callback(event::Event::WindowEvent {
                        window_id,
                        event: event::WindowEvent::CursorMoved {
                            device_id,
                            position: PhysicalPosition::new(pointer.x() as f64, pointer.y() as f64),
                        },
                    }, self.window_target());
                    callback(event::Event::WindowEvent {
                        window_id,
                        event: event::WindowEvent::MouseWheel {
                            device_id,
                            delta: event::MouseScrollDelta::LineDelta(
                                pointer.axis_value(android_activity::input::Axis::Hscroll),
                                pointer.axis_value(android_activity::input::Axis::Vscroll)),
                            phase: event::TouchPhase::Moved,
                        },
                    }, self.window_target());
                    return InputStatus::Handled;
                }
                let phase = match motion_event.action() {'''
if 'Android touchpads report' not in s:
    assert old in s
    s=s.replace(old,new,1)
# Mouse/trackpad motion must never become egui Touch events (which select text).
mouse = (Path(__file__).parent/'pointer_input.rs.inc').read_text('utf-8')
if '// ArtCraft pointer source routing' not in s:
    s=s.replace('                let phase = match motion_event.action() {', mouse+'\n                let phase = match motion_event.action() {',1)
s=s.replace('self.app.show_soft_input(false);','self.app.show_soft_input(true);')
# NativeActivity input is window-relative; CraftActivity renders into a child
# SurfaceView below the system caption. Use the live content origin, not a
# device-specific caption height. Only positions change, never gesture deltas.
anchor = '                let device_id = event::DeviceId(DeviceId(motion_event.device_id()));'
assert s.count(anchor) == 1
s=s.replace(anchor, anchor+'''
                let content = android_app.content_rect();
                let surface_position = |x: f64, y: f64| {
                    PhysicalPosition::new(x - content.left as f64, y - content.top as f64)
                };
''', 1)
assert s.count('PhysicalPosition::new(pointer.x() as f64, pointer.y() as f64)') == 2
s=s.replace('PhysicalPosition::new(pointer.x() as f64, pointer.y() as f64)',
            'surface_position(pointer.x() as f64, pointer.y() as f64)')
tap = '0 | 3 => event::WindowEvent::CursorMoved { device_id, position: PhysicalPosition::new(e[1] as f64, e[2] as f64) }'
assert tap in s
s=s.replace(tap, tap.replace('PhysicalPosition::new', 'surface_position'))
touch = 'PhysicalPosition { x: pointer.x() as _, y: pointer.y() as _ }'
assert touch in s
s=s.replace(touch, 'surface_position(pointer.x() as f64, pointer.y() as f64)')
s=s.replace('self.app.hide_soft_input(true);','self.app.hide_soft_input(false);')
# Unicode IME commits have characters only on the Java KeyEvent, not AKeyEvent.
old_key = '            InputEvent::KeyEvent(key) => {\n                match key.key_code() {'
new_key = '            InputEvent::KeyEvent(key) => {\n                if key.action() == KeyAction::Multiple { return InputStatus::Unhandled; }\n                match key.key_code() {'
if old_key in s:
    s=s.replace(old_key,new_key,1)
# SDK recognizers and JNI are shared across all native Android entry points.
sdk = Path(__file__).parent/'touchpad_sdk.rs'
sdk_target=dest/'src/platform_impl/android/touchpad_sdk.rs'
if not sdk_target.exists() or sdk_target.read_bytes()!=sdk.read_bytes():
    sdk_target.write_bytes(sdk.read_bytes())
s=s.replace('mod keycodes;', 'mod keycodes;\nmod touchpad_sdk;')
if p.read_text('utf-8') != s:
    p.write_text(s,'utf-8')
cargo=dest/'Cargo.toml'
s=cargo.read_text('utf-8')
s=s.replace('tracing = { version = "0.1.40", default-features = false }', 'tracing = { version = "0.1.40", default-features = false, features = ["log"] }')
if 'jni = "0.21.1"' not in s:
    s=s.replace('android-activity = "0.6.0"', 'android-activity = "0.6.0"\njni = "0.21.1"')
if cargo.read_text('utf-8') != s:
    cargo.write_text(s,'utf-8')
# Every Android entry point must use the same input backend, including PhotoCraft
# and the launcher. Patch both crates.io and direct git dependants.
import os
base=root/'scripts/artcraft-android'
for manifest in [base/'Cargo.toml', *sorted((base/'suite').glob('*/Cargo.toml'))]:
    s=manifest.read_text('utf-8')
    relative=os.path.relpath(dest, manifest.parent).replace('\\','/')
    replacement=f'winit = {{ path = "{relative}" }}'
    pinned='winit = { git = "https://github.com/storytold/winit.git", rev = "a517b3846ee8457d84ee4975110bfda6797c44dc" }'
    s=s.replace(pinned,replacement)
    if '[patch."https://github.com/storytold/winit.git"]' not in s:
        s+='\n[patch."https://github.com/storytold/winit.git"]\n'+replacement+'\n'
    if manifest.read_text('utf-8') != s:
        manifest.write_text(s,'utf-8')
