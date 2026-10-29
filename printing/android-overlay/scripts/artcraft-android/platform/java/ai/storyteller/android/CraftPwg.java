package ai.storyteller.android;

import android.graphics.*;
import android.graphics.pdf.PdfRenderer;
import android.os.ParcelFileDescriptor;
import java.io.*;
import java.nio.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import java.util.concurrent.atomic.AtomicBoolean;

/** PWG 5102.4 / RaS2: big-endian header, pixel RLE, bounded-height raster bands. */
final class CraftPwg {
    interface Progress {void update(String text);}
    static byte[] header(CraftPrintSettings s,int total){
        ByteBuffer h=ByteBuffer.allocate(1796).order(ByteOrder.BIG_ENDIAN);
        putString(h,0,"PwgRaster");putString(h,128,s.mediaType);putString(h,1732,s.paper.key);
        int w=Math.round(s.paper.width*360f/2540),height=Math.round(s.paper.height*360f/2540),channels=s.mono?1:3;
        h.putInt(276,360);h.putInt(280,360);h.putInt(340,1);h.putInt(352,Math.round(s.paper.width*72f/2540));h.putInt(356,Math.round(s.paper.height*72f/2540));
        h.putInt(372,w);h.putInt(376,height);h.putInt(384,8);h.putInt(388,channels*8);h.putInt(392,w*channels);h.putInt(400,s.mono?18:19);h.putInt(420,channels);
        h.putInt(452,total);h.putInt(456,1);h.putInt(460,1);h.putInt(472,w);h.putInt(476,height);h.putInt(480,0xffffff);
        return h.array();
    }
    private static void putString(ByteBuffer b,int offset,String value){byte[] bytes=value.getBytes(StandardCharsets.US_ASCII);for(int i=0;i<Math.min(63,bytes.length);i++)b.put(offset+i,bytes[i]);}
    static boolean same(byte[] row,int a,int b,int components){for(int i=0;i<components;i++)if(row[a+i]!=row[b+i])return false;return true;}
    static void writeRow(OutputStream out,byte[] row,int components)throws IOException{
        out.write(0); // This row occurs exactly once.
        int pos=0;
        while(pos<row.length){
            int repeated=1;while(repeated<128&&pos+(repeated+1)*components<=row.length&&same(row,pos,pos+repeated*components,components))repeated++;
            if(repeated>=2){out.write(repeated-1);out.write(row,pos,components);pos+=repeated*components;}
            else {int first=pos;pos+=components;int count=1;
                while(count<128&&pos<row.length){if(pos+components<row.length&&same(row,pos,pos+components,components))break;pos+=components;count++;}
                out.write(257-count);out.write(row,first,count*components);
            }
        }
    }
    static Matrix layout(CraftPrintSettings s,int width,int height){
        Matrix m=new Matrix();
        if(s.landscape){m.setValues(new float[]{0,-1,width,1,0,0,0,0,1});}
        if(s.rotate180)m.postRotate(180,width/2f,height/2f);
        if(s.mirror)m.postScale(-1,1,width/2f,height/2f);
        return m;
    }
    static Bitmap render(PdfRenderer pdf,CraftPrintSettings s,List<Integer> pages,int width,int height,int y,int bandHeight)throws IOException{
        Bitmap bitmap=Bitmap.createBitmap(width,bandHeight,Bitmap.Config.ARGB_8888);bitmap.eraseColor(Color.WHITE);
        try{
            float logicalW=s.landscape?height:width,logicalH=s.landscape?width:height;
            int cols=s.nup==4?2:1,rows=s.nup==1?1:2;
            float margin=s.marginMm*width/(s.paper.width/100f),cellW=(logicalW-2*margin)/cols,cellH=(logicalH-2*margin)/rows;
            Matrix transform=layout(s,width,height);transform.postTranslate(0,-y);
            for(int slot=0;slot<pages.size();slot++)try(PdfRenderer.Page page=pdf.openPage(pages.get(slot))){
                float left=margin+(slot%cols)*cellW,top=margin+(slot/cols)*cellH;
                float scale=s.fit?Math.min(cellW/page.getWidth(),cellH/page.getHeight()):(width/(s.paper.width*72f/2540))*s.scalePercent/100;
                Matrix local=new Matrix();local.setScale(scale,scale);local.postTranslate(left+(cellW-page.getWidth()*scale)/2,top+(cellH-page.getHeight()*scale)/2);
                Matrix finalMatrix=new Matrix();finalMatrix.setConcat(transform,local);
                RectF clip=new RectF(left,top,left+cellW,top+cellH);transform.mapRect(clip);Rect bounds=new Rect();clip.roundOut(bounds);
                if(bounds.intersect(0,0,width,bandHeight))page.render(bitmap,bounds,finalMatrix,PdfRenderer.Page.RENDER_MODE_FOR_PRINT);
            }
            if(!s.watermark.isEmpty()){
                Canvas canvas=new Canvas(bitmap);canvas.concat(transform);Paint paint=new Paint(Paint.ANTI_ALIAS_FLAG);paint.setColor(0x60808080);paint.setTextAlign(Paint.Align.CENTER);paint.setTextSize(logicalW/18);
                canvas.rotate(-35,logicalW/2,logicalH/2);canvas.drawText(s.watermark,logicalW/2,logicalH/2,paint);
            }
            if(s.mono){Canvas canvas=new Canvas(bitmap);Paint paint=new Paint();ColorMatrix cm=new ColorMatrix();cm.setSaturation(0);paint.setColorFilter(new ColorMatrixColorFilter(cm));canvas.drawBitmap(bitmap,0,0,paint);}
            return bitmap;
        }catch(Exception e){bitmap.recycle();throw new IOException("Не удалось подготовить страницу",e);}
    }
    static void write(File pdfFile,File output,CraftPrintSettings s,AtomicBoolean cancel,Progress progress)throws Exception{
        boolean complete=false;
        try(ParcelFileDescriptor fd=ParcelFileDescriptor.open(pdfFile,ParcelFileDescriptor.MODE_READ_ONLY);PdfRenderer pdf=new PdfRenderer(fd);OutputStream out=new BufferedOutputStream(new FileOutputStream(output),65536)){
            List<List<Integer>> sheets=s.sheets(pdf.getPageCount());out.write(new byte[]{'R','a','S','2'});
            int width=Math.round(s.paper.width*360f/2540),height=Math.round(s.paper.height*360f/2540),components=s.mono?1:3;
            int[] pixels=new int[width];byte[] row=new byte[width*components];
            for(int sheet=0;sheet<sheets.size();sheet++){
                progress.update("Подготовка листа "+(sheet+1)+" из "+sheets.size());out.write(header(s,sheets.size()));
                for(int y=0;y<height;y+=128){
                    if(cancel.get())throw new IOException("Подготовка отменена");int rows=Math.min(128,height-y);
                    Bitmap bitmap=render(pdf,s,sheets.get(sheet),width,height,y,rows);
                    try{for(int line=0;line<rows;line++){
                        bitmap.getPixels(pixels,0,width,0,line,width,1);
                        for(int x=0;x<width;x++){int p=pixels[x];if(s.mono)row[x]=(byte)(p&255);else {row[x*3]=(byte)(p>>>16);row[x*3+1]=(byte)(p>>>8);row[x*3+2]=(byte)p;}}
                        writeRow(out,row,components);
                    }}finally{bitmap.recycle();}
                }
            }
            complete=true;
        }finally{if(!complete)output.delete();}
    }
}
