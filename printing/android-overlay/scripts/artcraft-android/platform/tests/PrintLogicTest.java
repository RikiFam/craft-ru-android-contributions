package ai.storyteller.android;
import java.io.*;
import java.util.*;
import java.nio.*;

public final class PrintLogicTest {
    static void require(boolean value,String text){if(!value)throw new AssertionError(text);}
    static byte[] decodeRow(byte[] encoded,int channels)throws Exception{
        ByteArrayOutputStream result=new ByteArrayOutputStream();DataInputStream in=new DataInputStream(new ByteArrayInputStream(encoded));require(in.readUnsignedByte()==0,"row repeat");
        while(in.available()>0){int count=in.readUnsignedByte();if(count<128){byte[] pixel=new byte[channels];in.readFully(pixel);for(int i=0;i<=count;i++)result.write(pixel);}else {int n=(257-count)*channels;byte[] pixels=new byte[n];in.readFully(pixels);result.write(pixels);}}return result.toByteArray();
    }
    public static void main(String[] args)throws Exception{
        Random random=new Random(5721);
        for(int channels:new int[]{1,3})for(int pixels:new int[]{1,2,127,128,129,255,2976})for(int mode=0;mode<3;mode++){
            byte[] row=new byte[channels*pixels];if(mode==0)random.nextBytes(row);if(mode==1)Arrays.fill(row,(byte)255);if(mode==2)for(int i=0;i<row.length;i++)row[i]=(byte)((i/(channels*64))%2);
            ByteArrayOutputStream encoded=new ByteArrayOutputStream();CraftPwg.writeRow(encoded,row,channels);require(Arrays.equals(row,decodeRow(encoded.toByteArray(),channels)),"PWG RLE roundtrip");
        }
        CraftPrintSettings s=new CraftPrintSettings();s.range="1-3, 5";s.parity=1;s.reverse=true;require(s.pages(5).equals(Arrays.asList(4,2,0)),"page range/parity/reverse");s.copies=2;s.nup=2;
        try{s.sheets(5);throw new AssertionError("Ambiguous manual duplex/nup accepted");}catch(IllegalArgumentException expected){}
        s.parity=0;s.range="1,3,5";
        require(s.sheets(5).toString().equals("[[4, 2], [0], [4, 2], [0]]"),"collated copies");s.collate=false;require(s.sheets(5).toString().equals("[[4, 2], [4, 2], [0], [0]]"),"uncollated copies");
        s.range="0-2";try{s.pages(5);throw new AssertionError("invalid range accepted");}catch(IllegalArgumentException expected){}
        ByteBuffer h=ByteBuffer.wrap(CraftPwg.header(s,4)).order(ByteOrder.BIG_ENDIAN);require(h.capacity()==1796,"header length");require(h.getInt(276)==360&&h.getInt(372)==2976&&h.getInt(376)==4209,"header geometry");require(h.getInt(392)==8928&&h.getInt(400)==19&&h.getInt(452)==4,"raster encoding fields");
        byte[] status=CraftMaintenance.join(CraftMaintenance.ascii("\0@BDC ST2\r\n"),CraftMaintenance.hex("0600010104020100"));
        require(CraftMaintenance.parse(status).ready(),"Epson ST2 idle fixture");status[15]=7;require(!CraftMaintenance.parse(status).ready(),"Cleaning must not be ready");
        for(int cut=0;cut<status.length;cut++){try{CraftMaintenance.parse(Arrays.copyOf(status,cut));throw new AssertionError("Truncated status accepted at "+cut);}catch(IOException expected){}}
        try{CraftMaintenance.ack(new ByteArrayInputStream(new byte[]{1}));throw new AssertionError("LPR rejection accepted");}catch(IOException expected){}
        try{CraftMaintenance.ack(new ByteArrayInputStream(new byte[0]));throw new AssertionError("LPR EOF accepted");}catch(IOException expected){}
        // Published Epson REMOTE1 wire command: CH, length 2 LE, group 0.
        byte[] cleaning=CraftMaintenance.command(true);require(cleaning.length<256,"bounded maintenance command");
        String encoded=Arrays.toString(cleaning);require(encoded.contains("67, 72, 2, 0, 0, 0"),"normal CH command");
        require(!encoded.contains("68, 65, 4, 0"),"no alignment writes");
        System.out.println("PASS: pixel RLE, page selection, copies/collation, PWG header, Epson status truncation and LPR rejection");
    }
}
