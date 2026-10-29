package ai.storyteller.android;

/** On-device read-only/Validate-Job smoke check. Does not create or send jobs. */
public final class PrinterProbe {
    public static void main(String[] args)throws Exception {
        CraftIpp printer=new CraftIpp(args[0],null);CraftIpp.Reply caps=printer.capabilities();
        System.out.println("printer="+caps.text("printer-make-and-model","unknown"));
        System.out.println("certificate="+printer.pin);
        System.out.println("raster="+caps.values("pwg-raster-document-resolution-supported"));
        System.out.println("queue="+caps.number("queued-job-count",-1));
        System.out.println("maintenance="+CraftMaintenance.state(args[0]).text());
        CraftPrintSettings settings=new CraftPrintSettings();
        for(int quality:new int[]{4,5})for(boolean mono:new boolean[]{false,true}) {
            settings.quality=quality;settings.mono=mono;
            try {printer.validate(settings);System.out.println("VALID plain A4 quality="+quality+" mono="+mono);}
            catch(Exception e){System.out.println("REJECTED plain A4 quality="+quality+" mono="+mono+" "+e);}
        }
        settings.paper=CraftPrintSettings.PAPERS[3];settings.mediaType="photographic-glossy";settings.mono=false;
        try{printer.validate(settings);System.out.println("VALID glossy 10x15 high color");}catch(Exception e){System.out.println("REJECTED glossy "+e);}
        CraftIpp wrongPin=new CraftIpp(args[0],"000000");
        try{wrongPin.capabilities();throw new AssertionError("Changed certificate accepted");}catch(javax.net.ssl.SSLException expected){System.out.println("PASS changed certificate rejected");}
        System.out.println("queue_after="+printer.capabilities().number("queued-job-count",-1));
        if(args.length>1&&"--test-cancel".equals(args[1])){
            CraftPrintSettings cancelSettings=new CraftPrintSettings();int id=printer.create(cancelSettings,"ArtCraft cancellation test - no document");
            printer.cancel(id);int state=printer.job(id).number("job-state",0);
            if(state!=7)throw new AssertionError("Expected canceled job, got "+state);
            System.out.println("PASS cancel empty job="+id+" state="+state+"; no document sent");
        }
    }
}
