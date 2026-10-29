"""Reproducible Android-only fixes on the pinned winit revision; cache stays untouched."""
from pathlib import Path
import shutil
root=Path(__file__).resolve().parents[3]
source=root/'tmp/artcraft/android/winit-source'
dest=root/'tmp/artcraft/android/vendor/winit'
if not dest.exists():
    shutil.copytree(source,dest,ignore=shutil.ignore_patterns('.git','target'))
p=dest/'src/platform_impl/android/mod.rs'
s=p.read_text('utf-8')
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
s=s.replace('self.app.show_soft_input(false);','self.app.show_soft_input(true);')
s=s.replace('self.app.hide_soft_input(true);','self.app.hide_soft_input(false);')
# Unicode IME commits have characters only on the Java KeyEvent, not AKeyEvent.
old_key = '            InputEvent::KeyEvent(key) => {\n                match key.key_code() {'
new_key = '            InputEvent::KeyEvent(key) => {\n                if key.action() == KeyAction::Multiple { return InputStatus::Unhandled; }\n                match key.key_code() {'
if old_key in s:
    s=s.replace(old_key,new_key,1)
if p.read_text('utf-8') != s:
    p.write_text(s,'utf-8')
manifest=root/'scripts/artcraft-android/suite/wordcraft/Cargo.toml'
s=manifest.read_text('utf-8')
s=s.replace('winit = { git = "https://github.com/storytold/winit.git", rev = "a517b3846ee8457d84ee4975110bfda6797c44dc" }','winit = { path = "../../../../tmp/artcraft/android/vendor/winit" }')
if '[patch."https://github.com/storytold/winit.git"]' not in s:
    s+='\n[patch."https://github.com/storytold/winit.git"]\nwinit = { path = "../../../../tmp/artcraft/android/vendor/winit" }\n'
if manifest.read_text('utf-8') != s:
    manifest.write_text(s,'utf-8')
