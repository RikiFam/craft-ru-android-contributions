package ai.storyteller.android;

import android.content.Context;
import android.graphics.*;
import android.graphics.pdf.PdfRenderer;
import android.os.*;
import android.print.*;
import android.print.pdf.PrintedPdfDocument;
import java.io.*;
import java.util.*;
import java.util.concurrent.*;

/** Android owns printer discovery and options. Only selected pages are rendered. */
final class CraftPrintAdapter extends PrintDocumentAdapter {
    private final Context context;
    private final File source;
    private final ExecutorService worker = Executors.newSingleThreadExecutor();
    private PrintAttributes attributes;
    private int count;
    CraftPrintAdapter(Context context, File source) { this.context=context; this.source=source; }
    @Override public void onLayout(PrintAttributes oldAttributes, PrintAttributes newAttributes,
            CancellationSignal cancel, LayoutResultCallback callback, Bundle extras) {
        attributes = newAttributes;
        worker.execute(() -> {
            if (cancel.isCanceled()) { callback.onLayoutCancelled(); return; }
            try (ParcelFileDescriptor fd=ParcelFileDescriptor.open(source,ParcelFileDescriptor.MODE_READ_ONLY);
                 PdfRenderer pdf=new PdfRenderer(fd)) {
                count=pdf.getPageCount();
                PrintDocumentInfo info=new PrintDocumentInfo.Builder(source.getName())
                    .setContentType(PrintDocumentInfo.CONTENT_TYPE_DOCUMENT).setPageCount(count).build();
                if (cancel.isCanceled()) callback.onLayoutCancelled();
                else callback.onLayoutFinished(info,!newAttributes.equals(oldAttributes));
            } catch (Exception e) { callback.onLayoutFailed("Не удалось подготовить печать: "+e.getMessage()); }
        });
    }
    @Override public void onWrite(PageRange[] ranges, ParcelFileDescriptor destination,
            CancellationSignal cancel, WriteResultCallback callback) {
        final PrintAttributes options=attributes;
        worker.execute(() -> {
            PrintedPdfDocument output=new PrintedPdfDocument(context,options);
            try (ParcelFileDescriptor fd=ParcelFileDescriptor.open(source,ParcelFileDescriptor.MODE_READ_ONLY);
                 PdfRenderer pdf=new PdfRenderer(fd);
                 OutputStream stream=new FileOutputStream(destination.getFileDescriptor())) {
                List<PageRange> written=new ArrayList<>();
                for (int i=0;i<pdf.getPageCount();i++) {
                    if (cancel.isCanceled()) { callback.onWriteCancelled(); return; }
                    boolean selected=false;
                    for (PageRange range:ranges) if(i>=range.getStart()&&i<=range.getEnd()) selected=true;
                    if (!selected) continue;
                    android.graphics.pdf.PdfDocument.Page page=output.startPage(i);
                    Rect area=output.getPageContentRect();
                    // 300 DPI, bounded to prevent large-format printers exhausting tablet memory.
                    float dpi=Math.min(300f,options.getResolution()==null?300f:Math.min(options.getResolution().getHorizontalDpi(),options.getResolution().getVerticalDpi()));
                    float scale=Math.min(dpi/72f,4096f/Math.max(area.width(),area.height()));
                    int width=Math.max(1,Math.round(area.width()*scale)), height=Math.max(1,Math.round(area.height()*scale));
                    Bitmap bitmap=Bitmap.createBitmap(width,height,Bitmap.Config.ARGB_8888);
                    try (PdfRenderer.Page input=pdf.openPage(i)) {
                        bitmap.eraseColor(Color.WHITE);
                        float fit=Math.min((float)width/input.getWidth(),(float)height/input.getHeight());
                        Matrix matrix=new Matrix();matrix.setScale(fit,fit);
                        matrix.postTranslate((width-input.getWidth()*fit)/2,(height-input.getHeight()*fit)/2);
                        input.render(bitmap,null,matrix,PdfRenderer.Page.RENDER_MODE_FOR_PRINT);
                        Paint paint=new Paint(Paint.FILTER_BITMAP_FLAG);
                        if(options.getColorMode()==PrintAttributes.COLOR_MODE_MONOCHROME) {
                            ColorMatrix mono=new ColorMatrix();mono.setSaturation(0);paint.setColorFilter(new ColorMatrixColorFilter(mono));
                        }
                        page.getCanvas().drawBitmap(bitmap,null,area,paint);
                    } finally { bitmap.recycle(); }
                    output.finishPage(page);
                    written.add(new PageRange(i,i));
                }
                if(cancel.isCanceled()) { callback.onWriteCancelled();return; }
                output.writeTo(stream);
                callback.onWriteFinished(written.toArray(new PageRange[0]));
            } catch (Exception e) { callback.onWriteFailed("Не удалось напечатать: "+e.getMessage()); }
            finally { output.close(); }
        });
    }
    @Override public void onFinish() { worker.shutdown(); }
}
