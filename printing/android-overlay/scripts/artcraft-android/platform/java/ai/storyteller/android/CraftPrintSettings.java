package ai.storyteller.android;

import java.util.*;

final class CraftPrintSettings {
    static final class Paper {
        final String key,label;final int width,height;
        Paper(String key,String label,int width,int height){this.key=key;this.label=label;this.width=width;this.height=height;}
        public String toString(){return label;}
    }
    static final Paper[] PAPERS={
        new Paper("iso_a4_210x297mm","A4 · 210 × 297 мм",21000,29700),
        new Paper("iso_a5_148x210mm","A5 · 148 × 210 мм",14800,21000),
        new Paper("iso_a6_105x148mm","A6 · 105 × 148 мм",10500,14800),
        new Paper("na_index-4x6_4x6in","Фото · 10 × 15 см",10160,15240),
        new Paper("na_5x7_5x7in","Фото · 13 × 18 см",12700,17780),
        new Paper("na_letter_8.5x11in","Letter · 216 × 279 мм",21590,27940),
        new Paper("iso_dl_110x220mm","Конверт DL · 110 × 220 мм",11000,22000)
    };
    static final String[] TYPES={"stationery","photographic-high-gloss","photographic","photographic-semi-gloss","photographic-glossy","photographic-matte","stationery-coated","envelope"};
    static final String[] TYPE_LABELS={"Обычная бумага","Фотобумага высшего качества, глянцевая","Фотобумага","Полуглянцевая фотобумага","Глянцевая фотобумага","Матовая фотобумага","Бумага с покрытием","Конверт"};
    Paper paper=PAPERS[0];String mediaType="stationery";int quality=4,copies=1,nup=1,parity=0;boolean mono=false,landscape=false,reverse=false,collate=true;
    float marginMm=3,scalePercent=100;boolean fit=true,rotate180=false,mirror=false;String range="",watermark="";
    List<Integer> pages(int count){
        TreeSet<Integer> selected=new TreeSet<>();
        if(range.trim().isEmpty())for(int i=0;i<count;i++)selected.add(i);
        else for(String token:range.split(",",-1)){
            String[] ends=token.trim().split("-",-1);if(ends.length<1||ends.length>2)throw new IllegalArgumentException("Страницы: например 1–3, 5 (используйте дефис)");
            int first,last;try{first=Integer.parseInt(ends[0].trim());last=ends.length==1?first:Integer.parseInt(ends[1].trim());}
            catch(NumberFormatException e){throw new IllegalArgumentException("Укажите номера страниц: например 1-3, 5");}
            if(first<1||last>count||last<first)throw new IllegalArgumentException("Допустимы страницы от 1 до "+count);
            for(int i=first-1;i<last;i++)selected.add(i);
        }
        List<Integer> result=new ArrayList<>();for(int i:selected)if(parity==0||(i+1)%2==(parity==1?1:0))result.add(i);
        if(reverse)Collections.reverse(result);if(result.isEmpty())throw new IllegalArgumentException("Нет страниц для печати");return result;
    }
    List<List<Integer>> sheets(int count){
        if(parity!=0&&nup!=1)throw new IllegalArgumentException("Для ручной печати с двух сторон выберите одну страницу на листе");
        List<Integer> pages=pages(count);List<List<Integer>> base=new ArrayList<>(),out=new ArrayList<>();
        for(int i=0;i<pages.size();i+=nup)base.add(new ArrayList<>(pages.subList(i,Math.min(i+nup,pages.size()))));
        if(collate)for(int c=0;c<copies;c++)out.addAll(base);else for(List<Integer> sheet:base)for(int c=0;c<copies;c++)out.add(sheet);
        if(out.size()>500)throw new IllegalArgumentException("За одно задание можно напечатать не более 500 листов");return out;
    }
}
