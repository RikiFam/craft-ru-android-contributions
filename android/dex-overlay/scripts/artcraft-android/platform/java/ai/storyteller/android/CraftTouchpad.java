package ai.storyteller.android;

import android.content.Context;
import android.os.Handler;
import android.os.Looper;
import android.view.GestureDetector;
import android.view.MotionEvent;
import android.view.ScaleGestureDetector;
import java.util.ArrayList;

/** Android's gesture recognizers for system-synthesized finger events from a pointer device.
 * Physical mouse buttons and touchscreen/stylus input remain on the native input path. */
final class CraftTouchpad {
    private final ArrayList<Float> output = new ArrayList<>();
    private final GestureDetector gestures;
    private final ScaleGestureDetector scale;
    private boolean scaled;
    private boolean active;
    CraftTouchpad(Context context) {
        Handler handler = new Handler(Looper.getMainLooper());
        gestures = new GestureDetector(context, new GestureDetector.SimpleOnGestureListener() {
            @Override public boolean onDown(MotionEvent e) { return true; }
            @Override public boolean onSingleTapUp(MotionEvent e) {
                if (!scaled) emit(0, e.getX(), e.getY());
                return true;
            }
            @Override public boolean onDoubleTapEvent(MotionEvent e) {
                if (e.getActionMasked() == MotionEvent.ACTION_UP && !scaled) emit(0, e.getX(), e.getY());
                return true;
            }
            @Override public boolean onScroll(MotionEvent first, MotionEvent last, float dx, float dy) {
                if (!scale.isInProgress() && !scaled) emit(1, -dx, -dy);
                return true;
            }
        }, handler);
        // Context clicks come from Android's secondary mouse button event.
        // Do not create delayed callbacks on a different thread from the native queue.
        gestures.setIsLongpressEnabled(false);
        scale = new ScaleGestureDetector(context, new ScaleGestureDetector.SimpleOnScaleGestureListener() {
            @Override public boolean onScaleBegin(ScaleGestureDetector detector) { scaled = true; return true; }
            @Override public boolean onScale(ScaleGestureDetector detector) {
                emit(2, (float)Math.log(detector.getScaleFactor()), 0); return true;
            }
        }, handler);
        scale.setQuickScaleEnabled(false);
    }
    private void emit(float kind, float x, float y) { output.add(kind); output.add(x); output.add(y); }
    synchronized float[] process(int device, int action, long down, long time, float[] data) {
        output.clear();
        int masked = action & MotionEvent.ACTION_MASK;
        if (masked == MotionEvent.ACTION_DOWN) active = true;
        if (!active) return new float[0]; // Up after a physical-button drag stays native.
        int count = data.length / 4;
        if (count == 0 || count > 32 || data.length % 4 != 0) return new float[0];
        MotionEvent.PointerProperties[] properties = new MotionEvent.PointerProperties[count];
        MotionEvent.PointerCoords[] coords = new MotionEvent.PointerCoords[count];
        for (int i = 0; i < count; i++) {
            properties[i] = new MotionEvent.PointerProperties();
            properties[i].id = (int)data[i*4]; properties[i].toolType = MotionEvent.TOOL_TYPE_FINGER;
            coords[i] = new MotionEvent.PointerCoords();
            coords[i].x = data[i*4+1]; coords[i].y = data[i*4+2]; coords[i].pressure = data[i*4+3];
        }
        MotionEvent event = MotionEvent.obtain(down, time, action, count, properties, coords,
                0, 0, 1f, 1f, device, 0, android.view.InputDevice.SOURCE_TOUCHSCREEN, 0);
        try {
            if (event.getActionMasked() == MotionEvent.ACTION_DOWN) {
                scaled = false;
                emit(3, event.getX(), event.getY());
            }
            scale.onTouchEvent(event);
            gestures.onTouchEvent(event);
            float[] result = new float[output.size()];
            for (int i = 0; i < result.length; i++) result[i] = output.get(i);
            return result;
        } finally {
            if (masked == MotionEvent.ACTION_UP || masked == MotionEvent.ACTION_CANCEL) active = false;
            event.recycle();
        }
    }
}
