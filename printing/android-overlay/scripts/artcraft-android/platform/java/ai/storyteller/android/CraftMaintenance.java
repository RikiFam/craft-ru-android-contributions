package ai.storyteller.android;

import java.io.*;
import java.net.*;
import java.nio.charset.StandardCharsets;
import java.util.*;

/** Independent implementation of Epson REMOTE1 NC/CH and read-only SNMP status.
 * No EEPROM writes, alignment values, firmware uploads or reset commands. */
final class CraftMaintenance {
    static final String STATUS_OID="1.3.6.1.4.1.1248.1.2.2.44.1.1.2.1.115.116.1.0.1";
    static byte[] join(byte[]... parts)throws IOException {ByteArrayOutputStream out=new ByteArrayOutputStream();for(byte[] p:parts)out.write(p);return out.toByteArray();}
    static byte[] hex(String s){s=s.replace(" ","");byte[] b=new byte[s.length()/2];for(int i=0;i<b.length;i++)b[i]=(byte)Integer.parseInt(s.substring(i*2,i*2+2),16);return b;}
    static byte[] ascii(String s){return s.getBytes(StandardCharsets.US_ASCII);}
    static byte[] tlv(int tag,byte[] value)throws IOException {ByteArrayOutputStream b=new ByteArrayOutputStream();b.write(tag);if(value.length<128)b.write(value.length);else{b.write(0x82);b.write(value.length>>>8);b.write(value.length);}b.write(value);return b.toByteArray();}
    static byte[] oid(String value)throws IOException {String[] nums=value.split("\\.");ByteArrayOutputStream b=new ByteArrayOutputStream();b.write(Integer.parseInt(nums[0])*40+Integer.parseInt(nums[1]));for(int i=2;i<nums.length;i++){int v=Integer.parseInt(nums[i]);int shift=0;while((v>>>(shift+7))!=0)shift+=7;for(;shift>=0;shift-=7)b.write(((v>>>shift)&127)|(shift>0?128:0));}return tlv(6,b.toByteArray());}
    static final class Ber {
        final byte[] data;int pos;Ber(byte[] data){this.data=data;}
        byte[] read(int tag)throws IOException {if(pos+2>data.length||(data[pos++]&255)!=tag)throw new IOException("Неверный ответ SNMP");int len=data[pos++]&255;if(len>127){int count=len&127;if(count==0||count>2||pos+count>data.length)throw new IOException("Неверная длина SNMP");len=0;while(count-->0)len=(len<<8)|(data[pos++]&255);}if(pos+len>data.length)throw new IOException("Оборванный ответ SNMP");byte[] v=Arrays.copyOfRange(data,pos,pos+len);pos+=len;return v;}
    }
    static byte[] snmp(String host)throws Exception {
        byte[] requestId=hex("01234567"), request=tlv(0x30,join(tlv(2,new byte[]{0}),tlv(4,ascii("public")),tlv(0xa0,join(tlv(2,requestId),tlv(2,new byte[]{0}),tlv(2,new byte[]{0}),tlv(0x30,tlv(0x30,join(oid(STATUS_OID),tlv(5,new byte[0]))))))));
        try(DatagramSocket socket=new DatagramSocket()){socket.connect(InetAddress.getByName(host),161);socket.setSoTimeout(5000);socket.send(new DatagramPacket(request,request.length));byte[] buffer=new byte[4096];DatagramPacket packet=new DatagramPacket(buffer,buffer.length);socket.receive(packet);
            Ber message=new Ber(new Ber(Arrays.copyOf(buffer,packet.getLength())).read(0x30));message.read(2);message.read(4);Ber reply=new Ber(message.read(0xa2));if(!Arrays.equals(requestId,reply.read(2)))throw new IOException("Неверный номер ответа SNMP");byte[] error=reply.read(2);for(byte b:error)if(b!=0)throw new IOException("Принтер не разрешил чтение состояния");reply.read(2);Ber binding=new Ber(new Ber(reply.read(0x30)).read(0x30));byte[] receivedOid=binding.read(6);if(!Arrays.equals(receivedOid,new Ber(oid(STATUS_OID)).read(6)))throw new IOException("Неверное поле SNMP");return binding.read(4);
        }
    }
    static final class State {
        int code=-1,error=-1;boolean ready(){return code==3||code==4;}
        String text(){switch(code){case 0:return "Ошибка принтера (код "+error+")";case 1:return "Печать контрольного листа";case 2:return "Печать";case 3:case 4:return "Готов";case 7:return "Очистка печатающей головки";default:return "Состояние Epson: "+code;}}
    }
    static State parse(byte[] data)throws IOException {
        byte[] magic=ascii("BDC ST2\r\n");int start=-1;outer:for(int i=0;i<=data.length-magic.length;i++){for(int j=0;j<magic.length;j++)if(data[i+j]!=magic[j])continue outer;start=i+magic.length;break;}
        if(start<0||start+2>data.length)throw new IOException("Принтер не вернул состояние Epson ST2");int len=(data[start]&255)|((data[start+1]&255)<<8);start+=2;if(start+len!=data.length)throw new IOException("Оборванное состояние Epson");State state=new State();while(start<data.length){if(start+2>data.length)throw new IOException("Неверное поле состояния");int tag=data[start++]&255,n=data[start++]&255;if(start+n>data.length)throw new IOException("Оборванное поле состояния");if(n>0){if(tag==1)state.code=data[start]&255;if(tag==2)state.error=data[start]&255;}start+=n;}if(state.code<0)throw new IOException("В ответе нет состояния принтера");return state;
    }
    static State state(String host)throws Exception {return parse(snmp(host));}
    static byte[] command(boolean clean)throws IOException {
        byte[] enter=hex("1b285208000052454d4f544531"),exit=hex("1b000000"),init=hex("1b401b40");
        byte[] prefix=join(hex("0000001b0140454a4c20313238342e340a40454a4c20202020200a"),init,enter);
        byte[] tail=join(init,enter,hex("4a45010000"),exit);
        if(clean)return join(prefix,hex("434802000000"),exit,tail);
        return join(prefix,hex("4a53040000000000505003000001fe455808000000000033ff01004e4302000000"),exit,hex("0d0a0d0a"),enter,hex("5649020000004c440000"),exit,new byte[]{12},tail);
    }
    static void ack(InputStream in)throws IOException {int value=in.read();if(value!=0)throw new IOException(value<0?"Принтер закрыл соединение":"Принтер отклонил команду обслуживания (LPR "+value+")");}
    static void send(OutputStream out,InputStream in,int code,String text)throws IOException {out.write(code);out.write(ascii(text+"\n"));out.flush();ack(in);}
    static void file(OutputStream out,InputStream in,int code,String name,byte[] bytes)throws IOException {send(out,in,code,bytes.length+" "+name);out.write(bytes);out.write(0);out.flush();ack(in);}
    /** Acknowledgement means queued, never proof of physical completion. Never auto-retry. */
    static void submit(CraftIpp printer,boolean clean)throws Exception {
        CraftIpp.Reply caps=printer.capabilities();String model=caps.text("printer-make-and-model","").toUpperCase(Locale.ROOT);
        if(!model.contains("EPSON")||!model.contains("L3250"))throw new IOException("Обслуживание настроено только для EPSON L3250");
        if(caps.number("queued-job-count",-1)!=0||caps.number("printer-state",0)!=3||!state(printer.host).ready())throw new IOException("Принтер занят или не готов. Дождитесь завершения текущей операции.");
        String id=String.format(Locale.ROOT,"%03d",System.currentTimeMillis()%1000)+"artcraft",data="dfA"+id,control="cfA"+id;
        byte[] cf=ascii("Hartcraft\nPartcraft\nJArtCraft "+(clean?"head cleaning":"nozzle check")+"\nl"+data+"\nU"+data+"\n");
        try(Socket socket=new Socket()){socket.connect(new InetSocketAddress(printer.host,515),6000);socket.setSoTimeout(10000);OutputStream out=socket.getOutputStream();InputStream in=socket.getInputStream();send(out,in,2,"PASSTHRU");file(out,in,2,control,cf);file(out,in,3,data,command(clean));}
    }
}
