package ai.storyteller.android;
import android.content.Context;
import android.os.Looper;
import android.os.SystemClock;
import android.view.MotionEvent;

/** Runs real Android SDK recognizers on synthetic sequences; never injects user input. */
public class TouchpadSdkTest {
    static boolean has(float[] result, int kind) {
        for (int i=0;i<result.length;i+=3) if (result[i]==kind) return true;
        return false;
    }
    static void require(boolean ok, String message) { if (!ok) throw new AssertionError(message); }
    static float[] point(float x, float y) { return new float[]{0,x,y,1}; }
    public static void main(String[] args) throws Exception {
        Looper.prepareMainLooper();
        Class<?> type=Class.forName("android.app.ActivityThread");
        Object thread=type.getMethod("systemMain").invoke(null);
        Context context=(Context)type.getMethod("getSystemContext").invoke(thread);
        long t=SystemClock.uptimeMillis();
        CraftTouchpad pad=new CraftTouchpad(context);
        require(pad.process(1,1,t,t,point(100,100)).length==0,"unpaired physical release must be ignored");
        pad.process(1,0,t,t,point(100,100));
        require(has(pad.process(1,1,t,t+40,point(100,100)),0),"tap must click");
        pad.process(1,0,t+100,t+100,point(100,100));
        require(has(pad.process(1,1,t+100,t+140,point(100,100)),0),"second tap must click too");
        for (boolean horizontal:new boolean[]{false,true}) {
            pad=new CraftTouchpad(context); t+=1000;
            pad.process(1,0,t,t,point(200,200));
            float[] move=pad.process(1,2,t,t+50,point(horizontal?400:200,horizontal?200:400));
            require(has(move,1)&&!has(move,0),"swipe must scroll without click");
            require(!has(pad.process(1,1,t,t+90,point(400,400)),0),"swipe end must not click");
        }
        pad=new CraftTouchpad(context); t+=1000;
        pad.process(1,0,t,t,point(100,200));
        pad.process(1,MotionEvent.ACTION_POINTER_DOWN|(1<<8),t,t+20,new float[]{0,100,200,1,1,400,200,1});
        boolean zoom=false;
        for (int i=1;i<=8;i++) {
            float[] move=pad.process(1,2,t,t+20+i*20,new float[]{0,100-i*20,200,1,1,400+i*20,200,1});
            zoom |= has(move,2); require(!has(move,0),"pinch must not click");
        }
        require(zoom,"pinch must zoom");
        pad.process(1,3,t,t+300,point(100,200));
        System.out.println("PASS: tap, vertical scroll, horizontal scroll, pinch, cancellation; real Android SDK");
    }
}
