import android.view.*;
import android.os.SystemClock;
public class ScrollProbe {
 public static void main(String[] args) throws Exception {
  Class<?> cls=Class.forName("android.hardware.input.InputManager");
  Object manager=cls.getMethod("getInstance").invoke(null);
  MotionEvent.PointerProperties prop=new MotionEvent.PointerProperties(); prop.id=0; prop.toolType=MotionEvent.TOOL_TYPE_MOUSE;
  MotionEvent.PointerCoords point=new MotionEvent.PointerCoords(); point.x=1200;point.y=800;
  point.setAxisValue(MotionEvent.AXIS_VSCROLL,Float.parseFloat(args[0]));
  long now=SystemClock.uptimeMillis();
  MotionEvent event=MotionEvent.obtain(now,now,MotionEvent.ACTION_SCROLL,1,new MotionEvent.PointerProperties[]{prop},new MotionEvent.PointerCoords[]{point},0,0,1,1,0,0,InputDevice.SOURCE_MOUSE,0);
  System.out.println(cls.getMethod("injectInputEvent",InputEvent.class,int.class).invoke(manager,event,2));
  event.recycle();
 }
}
