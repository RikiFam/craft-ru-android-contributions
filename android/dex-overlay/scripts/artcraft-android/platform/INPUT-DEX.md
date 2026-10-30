# Android input and DeX windows

The same backend is used by all 13 editors and the launcher. No device name,
Samsung-specific velocity threshold, or raw touchpad coordinate recognizer is used.

## Input paths

- `ACTION_SCROLL`: Android HSCROLL/VSCROLL axes become winit MouseWheel.
- Mouse buttons and hover: MouseInput / CursorMoved, never touchscreen selection.
- Finger events from a mouse-class source: Android GestureDetector and
  ScaleGestureDetector process the system-generated gesture stream. JNI passes
  pointer IDs, coordinates, pressure, action/index and original timestamps.
- SDK scroll distances become pixel wheel events; SDK scale factors become log
  deltas, matching egui-winit's exponential conversion. Tap and double tap generate
  ordinary mouse clicks. Context clicks use Android's secondary button events.
- Genuine touchscreen and stylus events retain the existing native touch path.
- System DeX shortcuts/gestures remain Android-owned.

Observed on SM-T970 / EF-DT970: pointer events use source `0x3002`, tool FINGER;
two-finger scrolling arrives as DOWN/MOVE/UP with zero button state and zero
wheel axes. Treating this as touchscreen dragging caused text selection.

## Window ownership

NativeActivity's default `Window.takeSurface` gives the renderer the whole window
surface, hiding the Java-rendered DeX caption. CraftActivity uses an ordinary child
SurfaceView for native rendering. The original NativeContentView is retained for
IME focus and content geometry. Android continues drawing its own caption, border,
minimize/maximize/close controls. No custom imitation of the DeX toolbar is drawn.

Manifests explicitly allow resizing, use unrestricted orientation, include
smallestScreenSize config changes and do not force a fullscreen theme.

## Verification

`tests/run-touchpad-tests.ps1` runs the actual Android SDK detectors on the tablet
with synthetic tap/double-tap, vertical/horizontal scroll, pinch and cancel streams.
The tests do not inject events into user documents.

On 2026-10-30 the user confirmed scrolling, pinch and DeX maximize worked in
WordCraft. `tmp/artcraft/android/dex-surface-check.png` shows the real caption after
the SurfaceView change. All-app installation hashes are recorded by
`update-input-dex.py` in `tmp/artcraft/android/input-dex-update/installed.json`.
An installation record is not proof that every app-specific canvas gesture was
physically exercised. No diagnostic pointer logging remains in release source.

## References

- https://developer.android.com/reference/android/view/InputDevice
- https://developer.android.com/reference/android/view/MotionEvent
- https://developer.android.com/reference/android/view/GestureDetector
- https://developer.android.com/reference/android/view/ScaleGestureDetector
- https://developer.samsung.com/samsung-dex/modify-optimizing.html
- Android 13 framework NativeActivity.java and inputflinger TouchInputMapper.cpp.

Export note: local device installation/inspection helpers and raw logs are omitted. run-touchpad-tests.ps1 compiles only by default; explicit -RunOnDevice -Adb -Serial enables on-device tests. No device tests run during preparation.
