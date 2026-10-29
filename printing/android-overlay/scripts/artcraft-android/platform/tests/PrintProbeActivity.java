package ai.storyteller.android;
import android.app.Activity;
import android.os.Bundle;
import android.graphics.*;
import android.graphics.pdf.PdfDocument;
import java.io.*;
import java.util.concurrent.atomic.AtomicBoolean;

/** Test-only APK, not compiled into the editor APKs. No automatic printing. */
public final class PrintProbeActivity extends Activity {
    @Override public void onCreate(Bundle state){super.onCreate(state);
        File file=new File(getCacheDir(),"direct-test.pdf");PdfDocument doc=new PdfDocument();
        try{
            PdfDocument.Page p=doc.startPage(new PdfDocument.PageInfo.Builder(595,842,1).create());Paint paint=new Paint(Paint.ANTI_ALIAS_FLAG);paint.setColor(Color.BLACK);paint.setTextSize(18);
            p.getCanvas().drawText("ArtCraft: прямая печать с планшета",35,55,paint);paint.setTextSize(12);p.getCanvas().drawText("Wi-Fi / A4 / обычная бумага / стандартное качество",35,80,paint);
            for(int i=0;i<4;i++){paint.setColor(new int[]{Color.RED,Color.GREEN,Color.BLUE,Color.BLACK}[i]);p.getCanvas().drawRect(35+i*65,100,90+i*65,115,paint);}
            paint.setColor(Color.BLACK);p.getCanvas().drawText("Одна проверочная страница. Прямая передача с планшета.",35,145,paint);
            doc.finishPage(p);try(OutputStream out=new FileOutputStream(file)){doc.writeTo(out);}
        }catch(Exception e){throw new RuntimeException(e);}finally{doc.close();}
        new Thread(()->{try{CraftPwg.write(file,new File(getCacheDir(),"test.pwg"),new CraftPrintSettings(),new AtomicBoolean(),s->{});}catch(Exception e){android.util.Log.e("PrintProbe","Raster failed",e);}},"raster-probe").start();
        new CraftPrinterDialog(this,file,"Проверка ArtCraft · один лист").show();
    }
}
