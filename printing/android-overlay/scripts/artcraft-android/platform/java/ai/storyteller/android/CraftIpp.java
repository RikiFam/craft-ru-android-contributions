package ai.storyteller.android;

import java.io.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.security.*;
import java.security.cert.*;
import java.util.*;
import java.util.concurrent.atomic.AtomicBoolean;
import javax.net.ssl.*;

/** IPPS only. A first, document-free capabilities request learns the certificate;
 * all following requests pin that certificate. No global TLS settings are changed. */
final class CraftIpp {
    final String host, uri;
    volatile String pin;
    private int sequence=1;
    CraftIpp(String host, String pin) {
        if (!host.matches("[0-9]{1,3}(\\.[0-9]{1,3}){3}")) throw new IllegalArgumentException("Укажите IPv4-адрес принтера");
        for(String part:host.split("\\.")) if(Integer.parseInt(part)>255) throw new IllegalArgumentException("Неверный IP-адрес");
        this.host=host; this.pin=pin; uri="ipps://"+host+":631/ipp/print";
    }
    static String fingerprint(X509Certificate cert) throws Exception {
        byte[] hash=MessageDigest.getInstance("SHA-256").digest(cert.getEncoded());
        StringBuilder out=new StringBuilder(); for(byte b:hash) out.append(String.format(Locale.ROOT,"%02x",b&255)); return out.toString();
    }
    static final class Message {
        final ByteArrayOutputStream bytes=new ByteArrayOutputStream();
        final DataOutputStream out=new DataOutputStream(bytes);
        Message(int operation,int id) throws IOException {out.writeShort(0x0101);out.writeShort(operation);out.writeInt(id);out.writeByte(1);}
        Message group(int tag)throws IOException {out.writeByte(tag);return this;}
        Message attr(int tag,String name,byte[] value)throws IOException {
            byte[] key=name.getBytes(StandardCharsets.UTF_8);out.writeByte(tag);out.writeShort(key.length);out.write(key);out.writeShort(value.length);out.write(value);return this;
        }
        Message text(int tag,String name,String value)throws IOException {return attr(tag,name,value.getBytes(StandardCharsets.UTF_8));}
        Message number(int tag,String name,int value)throws IOException {return attr(tag,name,new byte[]{(byte)(value>>>24),(byte)(value>>>16),(byte)(value>>>8),(byte)value});}
        Message bool(String name,boolean value)throws IOException{return attr(0x22,name,new byte[]{(byte)(value?1:0)});}
        Message member(String name)throws IOException{return text(0x4a,"",name);}
        Message collection(String name)throws IOException{return attr(0x34,name,new byte[0]);}
        Message endCollection()throws IOException{return attr(0x37,"",new byte[0]);}
        byte[] finish()throws IOException{out.writeByte(3);return bytes.toByteArray();}
    }
    static final class Reply {
        int status;
        final Map<String,List<Object>> attrs=new LinkedHashMap<>();
        List<Object> values(String key){return attrs.containsKey(key)?attrs.get(key):Collections.emptyList();}
        String text(String key,String fallback){List<Object> v=values(key);return v.isEmpty()?fallback:String.valueOf(v.get(0));}
        int number(String key,int fallback){List<Object> v=values(key);return v.isEmpty()||!(v.get(0) instanceof Integer)?fallback:(Integer)v.get(0);}
        boolean has(String key,Object value){return values(key).contains(value);}
        void exact()throws IOException {if(status!=0)throw new IOException("Принтер не принял параметры без замены (IPP "+Integer.toHexString(status)+"): "+text("status-message","проверьте бумагу, качество и цвет"));}
    }
    static Reply parse(byte[] data,int expectedId)throws IOException {
        DataInputStream in=new DataInputStream(new ByteArrayInputStream(data));in.readUnsignedShort();Reply reply=new Reply();reply.status=in.readUnsignedShort();
        if(in.readInt()!=expectedId)throw new IOException("Неверный номер ответа принтера");
        String name="";int depth=0;
        while(in.available()>0){int tag=in.readUnsignedByte();if(tag==3)break;if(tag<16)continue;
            int length=in.readUnsignedShort();byte[] key=new byte[length];in.readFully(key);if(length>0)name=new String(key,StandardCharsets.UTF_8);
            length=in.readUnsignedShort();byte[] value=new byte[length];in.readFully(value);
            if(tag==0x34){depth++;continue;}if(tag==0x37){depth--;continue;}if(depth>0)continue;
            Object decoded=new String(value,StandardCharsets.UTF_8);
            if((tag==0x21||tag==0x23)&&length==4)decoded=new DataInputStream(new ByteArrayInputStream(value)).readInt();
            else if(tag==0x22&&length==1)decoded=value[0]!=0;
            else if(tag==0x32&&length==9){DataInputStream v=new DataInputStream(new ByteArrayInputStream(value));decoded=v.readInt()+"x"+v.readInt()+":"+v.readUnsignedByte();}
            reply.attrs.computeIfAbsent(name,k->new ArrayList<>()).add(decoded);
        }
        if(reply.status>=0x400){
            if(reply.attrs.containsKey("print-quality"))throw new IOException("Это качество недоступно для выбранного типа бумаги при прямой печати. Для обычной бумаги выберите стандартное качество; высокое проверено с глянцевой фотобумагой.");
            throw new IOException("Принтер отклонил параметры (IPP "+Integer.toHexString(reply.status)+"): "+reply.text("status-message","проверьте тип и размер бумаги, цвет и состояние принтера"));
        }
        return reply;
    }
    private synchronized int next(){return sequence++;}
    private Message operation(int op,int id)throws IOException {
        return new Message(op,id).text(0x47,"attributes-charset","utf-8").text(0x48,"attributes-natural-language","ru")
            .text(0x45,"printer-uri",uri).text(0x42,"requesting-user-name","ArtCraft");
    }
    private Reply exchange(int id,Message message,File payload,AtomicBoolean cancel,boolean learn)throws Exception {
        if(pin==null&&!learn)throw new IOException("Сначала подключитесь к принтеру");
        final String expected=pin;final String[] observed={null};
        SSLContext tls=SSLContext.getInstance("TLS");
        tls.init(null,new TrustManager[]{new X509TrustManager(){
            public X509Certificate[] getAcceptedIssuers(){return new X509Certificate[0];}
            public void checkClientTrusted(X509Certificate[] chain,String auth)throws CertificateException{throw new CertificateException("Client TLS unsupported");}
            public void checkServerTrusted(X509Certificate[] chain,String auth)throws CertificateException{
                try{if(chain.length==0)throw new CertificateException("No certificate");String found=fingerprint(chain[0]);
                    if(expected!=null&&!expected.equals(found))throw new CertificateException("Сертификат принтера изменился. Подключение остановлено.");observed[0]=found;
                }catch(Exception e){throw new CertificateException(e);}
            }
        }},new SecureRandom());
        HttpsURLConnection connection=(HttpsURLConnection)new URL("https://"+host+":631/ipp/print").openConnection();
        connection.setSSLSocketFactory(tls.getSocketFactory());
        connection.setHostnameVerifier((hostname,session)->{try{return host.equals(hostname)&&observed[0]!=null&&observed[0].equals(fingerprint((X509Certificate)session.getPeerCertificates()[0]));}catch(Exception e){return false;}});
        connection.setConnectTimeout(12000);connection.setReadTimeout(30000);connection.setInstanceFollowRedirects(false);
        connection.setRequestMethod("POST");connection.setDoOutput(true);connection.setRequestProperty("Content-Type","application/ipp");
        byte[] prefix=message.finish();connection.setFixedLengthStreamingMode(prefix.length+(payload==null?0L:payload.length()));
        try {
            try(OutputStream out=connection.getOutputStream()){
                out.write(prefix);
                if(payload!=null)try(InputStream in=new FileInputStream(payload)){byte[] buffer=new byte[65536];int n;while((n=in.read(buffer))!=-1){if(cancel!=null&&cancel.get())throw new IOException("Передача отменена");out.write(buffer,0,n);}}
            }
            int code=connection.getResponseCode();if(code!=200)throw new IOException("Ответ принтера HTTP "+code);
            ByteArrayOutputStream data=new ByteArrayOutputStream();try(InputStream in=connection.getInputStream()){byte[] buffer=new byte[8192];int n;while((n=in.read(buffer))!=-1){if(data.size()+n>1048576)throw new IOException("Слишком большой ответ принтера");data.write(buffer,0,n);}}
            Reply reply=parse(data.toByteArray(),id);if(learn&&pin==null)pin=observed[0];return reply;
        }finally{connection.disconnect();}
    }
    Reply capabilities()throws Exception{int id=next();return exchange(id,operation(11,id).text(0x44,"requested-attributes","all").text(0x49,"document-format","image/pwg-raster"),null,null,true);}
    private Message options(Message m,CraftPrintSettings s)throws IOException {
        m.group(2).number(0x21,"copies",1).text(0x44,"sides","one-sided").number(0x23,"print-quality",s.quality)
            .text(0x44,"print-color-mode",s.mono?"monochrome":"color").text(0x44,"print-scaling","none");
        m.collection("media-col").member("media-size").collection("").member("x-dimension").number(0x21,"",s.paper.width)
            .member("y-dimension").number(0x21,"",s.paper.height).endCollection()
            .member("media-type").text(0x44,"",s.mediaType).member("media-source").text(0x44,"","main");
        for(String side:new String[]{"top","left","right","bottom"})m.member("media-"+side+"-margin").number(0x21,"",300);
        return m.endCollection();
    }
    void validate(CraftPrintSettings settings)throws Exception{int id=next();Message m=operation(4,id).text(0x49,"document-format","image/pwg-raster").bool("ipp-attribute-fidelity",true);exchange(id,options(m,settings),null,null,false).exact();}
    int create(CraftPrintSettings settings,String title)throws Exception{int id=next();Message m=operation(5,id).text(0x42,"job-name",title).bool("ipp-attribute-fidelity",true);Reply r=exchange(id,options(m,settings),null,null,false);r.exact();int job=r.number("job-id",0);if(job<=0)throw new IOException("Принтер не вернул номер задания — проверьте очередь перед повтором");return job;}
    void send(int job,File raster,AtomicBoolean cancel)throws Exception{int id=next();Message m=operation(6,id).number(0x21,"job-id",job).text(0x49,"document-format","image/pwg-raster").bool("last-document",true);exchange(id,m,raster,cancel,false).exact();}
    Reply job(int job)throws Exception{int id=next();return exchange(id,operation(9,id).number(0x21,"job-id",job),null,null,false);}
    void cancel(int job)throws Exception{int id=next();exchange(id,operation(8,id).number(0x21,"job-id",job),null,null,false).exact();}
}
