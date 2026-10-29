package ai.storyteller.android;

import android.app.*;
import android.content.*;
import android.graphics.Bitmap;
import android.graphics.Color;
import android.graphics.Typeface;
import android.graphics.pdf.PdfRenderer;
import android.os.ParcelFileDescriptor;
import android.text.InputType;
import android.view.*;
import android.widget.*;
import java.io.*;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicBoolean;
import org.json.*;

/** Shared, tablet-sized printer properties. The Windows driver is not required. */
final class CraftPrinterDialog {
    final Activity activity;final Context ui;final File source;final String title;
    final ExecutorService worker=Executors.newSingleThreadExecutor();final AtomicBoolean cancelled=new AtomicBoolean();
    final SharedPreferences prefs;final Dialog dialog;final List<View> controls=new ArrayList<>();
    CraftIpp printer;CraftIpp.Reply caps;volatile int jobId;int pageCount;boolean busy;
    TextView status;EditText address,copies,range,scale,watermark;Spinner paper,type,quality,color,orientation,nup,parity,margin,sizing;
    CheckBox collate,reverse,rotate,mirror;Button print,check,connect,preview;LinearLayout presets;
    List<CraftPrintSettings.Paper> papers=new ArrayList<>();List<String> types=new ArrayList<>();List<Integer> qualities=new ArrayList<>();
    CraftPrinterDialog(Activity activity,File source,String title){
        this.activity=activity;this.source=source;this.title=title;ui=new ContextThemeWrapper(activity,android.R.style.Theme_Material_Light_NoActionBar);
        prefs=activity.getSharedPreferences("direct-print",Context.MODE_PRIVATE);dialog=new Dialog(ui);dialog.setTitle("Свойства принтера");
    }
    int dp(int value){return Math.round(value*ui.getResources().getDisplayMetrics().density);}
    LinearLayout column(){LinearLayout v=new LinearLayout(ui);v.setOrientation(LinearLayout.VERTICAL);return v;}
    LinearLayout row(){LinearLayout v=new LinearLayout(ui);v.setOrientation(LinearLayout.HORIZONTAL);v.setGravity(Gravity.CENTER_VERTICAL);return v;}
    TextView label(String text){TextView v=new TextView(ui);v.setText(text);v.setTextColor(Color.rgb(25,25,25));v.setTextSize(15);v.setPadding(dp(5),dp(6),dp(5),dp(6));return v;}
    void heading(LinearLayout box,String text){TextView v=label(text);v.setTypeface(null,Typeface.BOLD);v.setTextSize(17);box.addView(v);}
    void line(LinearLayout box){View v=new View(ui);v.setBackgroundColor(0xffcccccc);LinearLayout.LayoutParams lp=new LinearLayout.LayoutParams(-1,dp(1));lp.setMargins(0,dp(8),0,dp(8));box.addView(v,lp);}
    Button button(String text,Runnable action){Button v=new Button(ui);v.setAllCaps(false);v.setText(text);v.setOnClickListener(w->action.run());return v;}
    Spinner choice(String... values){Spinner v=new Spinner(ui);setChoices(v,Arrays.asList(values));controls.add(v);return v;}
    void setChoices(Spinner spinner,List<?> values){ArrayAdapter<Object> adapter=new ArrayAdapter<>(ui,android.R.layout.simple_spinner_item,new ArrayList<Object>(values));adapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item);spinner.setAdapter(adapter);}
    EditText input(String value,boolean numeric){EditText v=new EditText(ui);v.setText(value);v.setSingleLine();v.setInputType(numeric?InputType.TYPE_CLASS_NUMBER:InputType.TYPE_CLASS_TEXT);controls.add(v);return v;}
    CheckBox box(String text,boolean checked){CheckBox v=new CheckBox(ui);v.setText(text);v.setChecked(checked);controls.add(v);return v;}
    void field(LinearLayout parent,String text,View widget){LinearLayout r=row();r.addView(label(text),new LinearLayout.LayoutParams(dp(185),-2));r.addView(widget,new LinearLayout.LayoutParams(0,dp(46),1));parent.addView(r);}
    ScrollView scroll(View view){ScrollView s=new ScrollView(ui);s.setFillViewport(true);s.addView(view);return s;}
    void show(){
        try(ParcelFileDescriptor fd=ParcelFileDescriptor.open(source,ParcelFileDescriptor.MODE_READ_ONLY);PdfRenderer pdf=new PdfRenderer(fd)){pageCount=pdf.getPageCount();}catch(Exception e){Toast.makeText(activity,"Не удалось открыть документ для печати",Toast.LENGTH_LONG).show();return;}
        LinearLayout root=column();root.setPadding(dp(16),dp(10),dp(16),dp(10));root.setBackgroundColor(0xfff3f3f3);
        heading(root,"Свойства: EPSON L3250 Series");
        LinearLayout connection=row();address=input(prefs.getString("address",""),false);address.setInputType(InputType.TYPE_CLASS_PHONE);
        connection.addView(label("Принтер Wi-Fi"));connection.addView(address,new LinearLayout.LayoutParams(dp(190),dp(44)));connect=button("Подключить",()->connect());connection.addView(connect);status=label("Подключение к принтеру…");connection.addView(status,new LinearLayout.LayoutParams(0,-2,1));root.addView(connection);
        LinearLayout body=row();presets=column();presets.setPadding(0,0,dp(14),0);heading(presets,"Предустановки печати");
        addPresetButton("Документ — стандартный",()->{quality.setSelection(0);color.setSelection(0);nup.setSelection(0);});
        addPresetButton("Фото — высокое качество",()->{int photo=types.indexOf("photographic-glossy");if(photo>=0)type.setSelection(photo);quality.setSelection(Math.max(0,qualities.indexOf(5)));color.setSelection(0);});
        addPresetButton("Документ — 2 на листе",()->nup.setSelection(1));
        addPresetButton("Документ — оттенки серого",()->color.setSelection(1));
        presets.addView(button("Сохранить предустановку…",()->savePreset()));presets.addView(button("Мои предустановки…",()->loadPreset()));
        line(presets);presets.addView(label("Документ: "+title+"\nСтраниц: "+pageCount));
        preview=button("Просмотр перед печатью",()->preview());presets.addView(preview);
        presets.addView(button("Восстановить настройки",()->defaults()));
        body.addView(scroll(presets),new LinearLayout.LayoutParams(dp(260),-1));
        TabHost tabs=new TabHost(ui);tabs.setId(android.R.id.tabhost);LinearLayout tabsBox=column();TabWidget tabWidget=new TabWidget(ui);tabWidget.setId(android.R.id.tabs);tabsBox.addView(tabWidget);
        FrameLayout content=new FrameLayout(ui);content.setId(android.R.id.tabcontent);tabsBox.addView(content,new LinearLayout.LayoutParams(-1,0,1));tabs.addView(tabsBox);tabs.setup();
        LinearLayout main=column();paper=choice();type=choice();quality=choice();color=choice("Цветной","Оттенки серого");orientation=choice("Книжная","Альбомная");
        field(main,"Размер бумаги",paper);field(main,"Ориентация",orientation);field(main,"Тип бумаги",type);field(main,"Качество",quality);field(main,"Цвет",color);line(main);
        parity=choice("Все страницы · одна сторона","Нечётные · первый проход","Чётные · второй проход");field(main,"Стороны печати",parity);
        main.addView(label("Две стороны: напечатайте нечётные страницы, переверните листы и выберите чётные. Автоматического переворота в L3250 нет."));
        nup=choice("1 страница на листе","2 страницы на листе","4 страницы на листе");field(main,"Размещение",nup);line(main);
        copies=input("1",true);field(main,"Копии",copies);collate=box("Разобрать по копиям",true);reverse=box("Обратный порядок",false);main.addView(collate);main.addView(reverse);
        range=input("",false);range.setHint("Все или, например, 1-3, 5");field(main,"Страницы документа",range);
        LinearLayout advanced=column();sizing=choice("Уместить на странице","Указать масштаб");field(advanced,"Масштабирование",sizing);scale=input("100",true);field(advanced,"Масштаб, %",scale);
        margin=choice("3 мм · поля принтера","10 мм","20 мм");field(advanced,"Поля на листе",margin);advanced.addView(label("Содержимое размещается по центру. При размещении нескольких страниц масштаб применяется к каждой странице."));line(advanced);
        rotate=box("Повернуть на 180°",false);mirror=box("Зеркально",false);advanced.addView(rotate);advanced.addView(mirror);watermark=input("",false);watermark.setHint("Например: КОПИЯ");field(advanced,"Водяной знак",watermark);line(advanced);
        advanced.addView(label("Тихий режим, двунаправленная печать и фирменная цветокоррекция Epson через это подключение недоступны. Цвет и качество задаются на вкладке «Главное»."));
        LinearLayout service=column();heading(service,"Состояние и обслуживание");
        service.addView(button("Проверка дюз печатающей головки…",()->confirmMaintenance(false)));
        service.addView(button("Очистка печатающей головки…",()->confirmMaintenance(true)));
        service.addView(button("Состояние Epson",()->maintenanceStatus()));
        service.addView(label("Проверка и обычная очистка запускаются прямо по Wi-Fi, без компьютера и других приложений. Перед запуском принтер должен быть свободен."));
        addTab(tabs,"main","Главное",main);addTab(tabs,"advanced","Дополнительные настройки",advanced);addTab(tabs,"service","Сервис",service);
        body.addView(tabs,new LinearLayout.LayoutParams(0,-1,1));root.addView(body,new LinearLayout.LayoutParams(-1,0,1));
        LinearLayout footer=row();check=button("Проверить параметры",()->validate());print=button("Печать…",()->confirmPrint());footer.addView(check);footer.addView(print);
        footer.addView(button("Печать Android…",()->{if(busy)return;((android.print.PrintManager)activity.getSystemService(Context.PRINT_SERVICE)).print(title,new CraftPrintAdapter(activity,source),null);}));
        footer.addView(button("Закрыть",()->{if(!busy)dialog.dismiss();else Toast.makeText(ui,"Сначала завершите или отмените задание",Toast.LENGTH_LONG).show();}));root.addView(footer);
        dialog.setContentView(root);dialog.setCancelable(false);dialog.setOnDismissListener(d->worker.shutdown());dialog.show();dialog.getWindow().setLayout((int)(activity.getResources().getDisplayMetrics().widthPixels*.96),(int)(activity.getResources().getDisplayMetrics().heightPixels*.94));dialog.getWindow().setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_STATE_ALWAYS_HIDDEN);
        connect();
    }
    void addTab(TabHost tabs,String id,String title,LinearLayout view){ScrollView sv=scroll(view);TextView indicator=label(title);indicator.setGravity(Gravity.CENTER);indicator.setPadding(dp(14),dp(14),dp(14),dp(14));indicator.setTypeface(null,Typeface.BOLD);indicator.setBackgroundResource(android.R.drawable.btn_default);tabs.addTab(tabs.newTabSpec(id).setIndicator(indicator).setContent(tag->sv));}
    void addPresetButton(String title,Runnable action){presets.addView(button(title,()->{if(!busy&&caps!=null)action.run();}));}
    void ui(Runnable action){activity.runOnUiThread(()->{if(!activity.isDestroyed()&&dialog.isShowing())action.run();});}
    void status(String text){ui(()->status.setText(text));}
    void error(Exception e){new AlertDialog.Builder(ui).setTitle("Не удалось выполнить действие").setMessage(e.getMessage()==null?e.toString():e.getMessage()).setPositiveButton("Закрыть",null).show();}
    void setBusy(boolean value){busy=value;for(View v:controls)v.setEnabled(!value);connect.setEnabled(!value);print.setEnabled(!value&&caps!=null);check.setEnabled(!value&&caps!=null);preview.setEnabled(!value&&caps!=null);}
    void connect(){
        if(busy)return;setBusy(true);status.setText("Проверка подключения…");String host=address.getText().toString().trim();
        worker.execute(()->{try{
            CraftIpp candidate=new CraftIpp(host,prefs.getString("pin."+host,null));CraftIpp.Reply found=candidate.capabilities();
            if(!found.has("document-format-supported","image/pwg-raster")||!found.has("pwg-raster-document-resolution-supported","360x360:3"))throw new IOException("Этот принтер не поддерживает используемый формат печати. Используйте «Печать Android…».");
            prefs.edit().putString("address",host).putString("pin."+host,candidate.pin).apply();
            ui(()->{printer=candidate;caps=found;papers.clear();for(CraftPrintSettings.Paper p:CraftPrintSettings.PAPERS)if(found.has("media-supported",p.key))papers.add(p);
                types.clear();List<String> labels=new ArrayList<>();for(int i=0;i<CraftPrintSettings.TYPES.length;i++)if(found.has("media-type-supported",CraftPrintSettings.TYPES[i])){types.add(CraftPrintSettings.TYPES[i]);labels.add(CraftPrintSettings.TYPE_LABELS[i]);}
                qualities.clear();List<String> qualityLabels=new ArrayList<>();for(int q:new int[]{4,5,3})if(found.has("print-quality-supported",q)){qualities.add(q);qualityLabels.add(q==4?"Стандартное":q==5?"Высокое":"Черновик");}
                setChoices(paper,papers);setChoices(type,labels);setChoices(quality,qualityLabels);status.setText(found.text("printer-make-and-model","Принтер")+" · "+state(found));setBusy(false);
            });
        }catch(Exception e){ui(()->{caps=null;status.setText("Нет подключения");setBusy(false);error(e);});}});
    }
    static String state(CraftIpp.Reply r){int s=r.number("printer-state",0);return(s==3?"Готов":s==4?"Печатает":s==5?"Остановлен":"Состояние неизвестно")+" · заданий: "+r.number("queued-job-count",0);}
    CraftPrintSettings read(){
        if(caps==null||papers.isEmpty()||types.isEmpty()||qualities.isEmpty())throw new IllegalArgumentException("Сначала подключитесь к принтеру");
        if(!address.getText().toString().trim().equals(printer.host))throw new IllegalArgumentException("Адрес изменён. Нажмите «Подключить».");
        CraftPrintSettings s=new CraftPrintSettings();s.paper=papers.get(paper.getSelectedItemPosition());s.mediaType=types.get(type.getSelectedItemPosition());s.quality=qualities.get(quality.getSelectedItemPosition());
        try{s.copies=Integer.parseInt(copies.getText().toString());}catch(NumberFormatException e){throw new IllegalArgumentException("Укажите число копий от 1 до 99");}if(s.copies<1||s.copies>99)throw new IllegalArgumentException("Укажите от 1 до 99 копий");s.mono=color.getSelectedItemPosition()==1;s.landscape=orientation.getSelectedItemPosition()==1;
        s.nup=new int[]{1,2,4}[nup.getSelectedItemPosition()];s.parity=parity.getSelectedItemPosition();s.collate=collate.isChecked();s.reverse=reverse.isChecked();s.rotate180=rotate.isChecked();s.mirror=mirror.isChecked();s.fit=sizing.getSelectedItemPosition()==0;
        try{s.scalePercent=Integer.parseInt(scale.getText().toString());}catch(NumberFormatException e){throw new IllegalArgumentException("Укажите масштаб от 10 до 400%");}if(s.scalePercent<10||s.scalePercent>400)throw new IllegalArgumentException("Масштаб: от 10 до 400%");s.marginMm=new int[]{3,10,20}[margin.getSelectedItemPosition()];s.watermark=watermark.getText().toString();s.range=range.getText().toString();s.sheets(pageCount);return s;
    }
    void validate(){if(busy)return;try{CraftPrintSettings s=read();setBusy(true);status.setText("Проверка параметров без печати…");worker.execute(()->{try{printer.validate(s);status("Принтер принял параметры. Бумага не расходуется.");}catch(Exception e){ui(()->error(e));}finally{ui(()->setBusy(false));}});}catch(Exception e){error(e);}}
    void preview(){if(busy)return;try{
        CraftPrintSettings s=read();List<List<Integer>> sheets=s.sheets(pageCount);final int[] index={0};final Bitmap[] current={null};
        LinearLayout content=column();TextView position=label("");ImageView image=new ImageView(ui);image.setAdjustViewBounds(true);content.addView(position);content.addView(image,new LinearLayout.LayoutParams(-1,dp(380)));LinearLayout nav=row();content.addView(nav);
        Runnable render=()->{try(ParcelFileDescriptor fd=ParcelFileDescriptor.open(source,ParcelFileDescriptor.MODE_READ_ONLY);PdfRenderer pdf=new PdfRenderer(fd)){
            int w=500,h=Math.round(w*(float)s.paper.height/s.paper.width);Bitmap next=CraftPwg.render(pdf,s,sheets.get(index[0]),w,h,0,h);image.setImageBitmap(next);if(current[0]!=null)current[0].recycle();current[0]=next;position.setText("Лист "+(index[0]+1)+" из "+sheets.size());
        }catch(Exception e){error(e);}};
        nav.addView(button("Предыдущий",()->{if(index[0]>0){index[0]--;render.run();}}));nav.addView(button("Следующий",()->{if(index[0]+1<sheets.size()){index[0]++;render.run();}}));
        AlertDialog view=new AlertDialog.Builder(ui).setTitle("Просмотр печати").setView(content).setPositiveButton("Закрыть",null).create();view.setOnDismissListener(d->{image.setImageDrawable(null);if(current[0]!=null)current[0].recycle();});view.show();render.run();
    }catch(Exception e){error(e);}}
    void confirmPrint(){if(busy)return;try{CraftPrintSettings s=read();new AlertDialog.Builder(ui).setTitle("Отправить на принтер?").setMessage("EPSON · "+printer.host+"\nЛистов: "+s.sheets(pageCount).size()+"\n"+s.paper.label+"\n"+(s.mono?"Оттенки серого":"Цветной")+" · "+(s.quality==5?"Высокое качество":"Стандартное качество")).setNegativeButton("Отмена",null).setPositiveButton("Печатать",(d,w)->submit(s)).show();}catch(Exception e){error(e);}}
    void submit(CraftPrintSettings s){
        setBusy(true);cancelled.set(false);jobId=0;ProgressDialog progress=new ProgressDialog(ui);progress.setTitle("Печать напрямую по Wi-Fi");progress.setMessage("Проверка параметров…");progress.setCancelable(false);progress.setButton(DialogInterface.BUTTON_NEGATIVE,"Отменить задание",(d,w)->{});progress.show();
        progress.getButton(DialogInterface.BUTTON_NEGATIVE).setOnClickListener(v->{cancelled.set(true);progress.setMessage("Отмена…");if(jobId>0)new Thread(()->{try{printer.cancel(jobId);status("Принтер принял отмену задания "+jobId);}catch(Exception e){status("Не удалось подтвердить отмену: "+e.getMessage());}},"print-cancel").start();});
        worker.execute(()->{File raster=null;try{
            printer.validate(s);raster=File.createTempFile("artcraft-print-",".pwg",activity.getCacheDir());
            CraftPwg.write(source,raster,s,cancelled,text->ui(()->progress.setMessage(text)));
            if(cancelled.get())throw new IOException("Печать отменена до отправки");
            jobId=printer.create(s,title);prefs.edit().putInt("job."+printer.host,jobId).apply();
            if(cancelled.get()){printer.cancel(jobId);throw new IOException("Задание отменено");}
            ui(()->progress.setMessage("Отправка задания "+jobId+"…"));printer.send(jobId,raster,cancelled);
            for(int i=0;i<120;i++){
                CraftIpp.Reply result=printer.job(jobId);int state=result.number("job-state",0);String message=jobState(result);status(message);ui(()->progress.setMessage(message));
                if(state>=7)return;if(cancelled.get()){printer.cancel(jobId);status("Принтер принял отмену задания "+jobId);return;}Thread.sleep(3000);
            }
            status("Задание "+jobId+" отправлено. Проверить: «Сервис → Состояние последнего задания».");
        }catch(Exception e){if(jobId>0&&cancelled.get())try{printer.cancel(jobId);}catch(Exception ignored){}
            final String message=(jobId>0?"Задание "+jobId+" могло поступить на принтер. Проверьте его состояние перед повторной отправкой.\n":"")+e.getMessage();ui(()->error(new IOException(message)));status("Печать остановлена — проверьте сообщение");
        }finally{if(raster!=null)raster.delete();ui(()->{progress.dismiss();setBusy(false);});}});
    }
    static String jobState(CraftIpp.Reply r){int state=r.number("job-state",0);String text=state==3?"В очереди":state==4?"Ожидает":state==5?"Печатается":state==6?"Приостановлено":state==7?"Отменено":state==8?"Завершено с ошибкой":state==9?"Принтер сообщил о завершении":"Состояние неизвестно";
        if(r.has("job-state-reasons","completed-successfully"))return "Печать успешно завершена";
        if(r.has("job-state-reasons","job-canceled-by-user"))return "Задание отменено пользователем";
        if(r.has("job-state-reasons","job-data-insufficient"))return text+" · ожидает данные документа";
        if(r.has("job-state-reasons","printer-stopped"))return text+" · принтер остановлен";
        if(r.has("job-state-reasons","document-format-error"))return text+" · ошибка формата документа";
        return text;
    }
    void lastJob(){if(busy||printer==null)return;int id=prefs.getInt("job."+printer.host,0);if(id==0){new AlertDialog.Builder(ui).setMessage("В этом приложении пока нет прямых заданий печати.").setPositiveButton("Закрыть",null).show();return;}
        setBusy(true);worker.execute(()->{try{CraftIpp.Reply r=printer.job(id);ui(()->{AlertDialog.Builder b=new AlertDialog.Builder(ui).setTitle("Задание "+id).setMessage(jobState(r)).setPositiveButton("Закрыть",null);if(r.number("job-state",9)<7)b.setNegativeButton("Отменить задание",(d,w)->worker.execute(()->{try{printer.cancel(id);status("Отмена задания принята");}catch(Exception e){ui(()->error(e));}}));b.show();});}catch(Exception e){ui(()->error(e));}finally{ui(()->setBusy(false));}});
    }
    void maintenanceStatus(){
        if(busy||printer==null)return;setBusy(true);worker.execute(()->{try{
            CraftIpp.Reply r=printer.capabilities();CraftMaintenance.State s=CraftMaintenance.state(printer.host);
            String message=r.text("printer-make-and-model","Epson")+"\nАдрес: "+printer.host+"\nСостояние: "+s.text()+"\nЗаданий в очереди: "+r.number("queued-job-count",0)+"\n\n"+prefs.getString("maintenance."+printer.host,"Обслуживание из этого приложения ещё не запускалось.");
            ui(()->new AlertDialog.Builder(ui).setTitle("Состояние Epson").setMessage(message).setPositiveButton("Закрыть",null).show());status(s.text());
        }catch(Exception e){ui(()->error(e));}finally{ui(()->setBusy(false));}});
    }
    void confirmMaintenance(boolean clean){
        if(busy||printer==null)return;
        String message=clean?"Очистка расходует чернила. Запускайте её, если на проверке дюз есть пропуски. Убедитесь, что уровень чернил выше нижней отметки. Во время очистки не выключайте принтер.":"Вставьте один лист обычной бумаги A4. Принтер напечатает собственный рисунок для проверки дюз. Будет использован один лист.";
        new AlertDialog.Builder(ui).setTitle(clean?"Очистить печатающую головку?":"Напечатать проверку дюз?").setMessage(message).setNegativeButton("Отмена",null).setPositiveButton(clean?"Начать очистку":"Напечатать один лист",(d,w)->maintenance(clean)).show();
    }
    void maintenance(boolean clean){
        if(busy||printer==null)return;setBusy(true);
        AlertDialog progress=new AlertDialog.Builder(ui).setTitle(clean?"Очистка головки":"Проверка дюз").setMessage("Проверка готовности принтера…").setCancelable(false).create();progress.show();
        worker.execute(()->{String result;try{
            CraftMaintenance.submit(printer,clean);
            prefs.edit().putString("maintenance."+printer.host,(clean?"Очистка":"Проверка дюз")+": команда принята; завершение ещё не подтверждено.").apply();
            boolean observed=false;long deadline=System.currentTimeMillis()+360000;result="Команда принята, но завершение не подтверждено. Проверьте состояние принтера; повторно автоматически ничего не отправлено.";
            while(System.currentTimeMillis()<deadline){CraftMaintenance.State state=CraftMaintenance.state(printer.host);if(state.code==0)throw new IOException(state.text());if(state.code==1||state.code==2||state.code==7||state.code==15)observed=true;
                String text=state.text()+"\nНе выключайте принтер.";ui(()->progress.setMessage(text));status(state.text());
                if(observed&&state.ready()){result=clean?"Принтер снова готов после обслуживания. Проверьте качество с помощью рисунка дюз.":"Принтер снова готов после печати контрольного рисунка. Проверьте лист: линии всех цветов должны быть без пропусков.";break;}
                if(!observed&&deadline-System.currentTimeMillis()<330000)break;Thread.sleep(1000);
            }
        }catch(Exception e){result="Не удалось подтвердить выполнение: "+e.getMessage()+"\nЕсли команда уже отправлена, принтер может продолжать работу. Автоматического повтора нет.";}
            final String message=result;prefs.edit().putString("maintenance."+printer.host,message).apply();ui(()->{progress.dismiss();setBusy(false);status.setText("Проверьте результат обслуживания");new AlertDialog.Builder(ui).setTitle("Обслуживание принтера").setMessage(message).setPositiveButton("Закрыть",null).show();});
        });
    }
    void defaults(){if(busy||caps==null)return;for(Spinner s:new Spinner[]{paper,type,quality,color,orientation,nup,parity,margin,sizing})s.setSelection(0);copies.setText("1");scale.setText("100");range.setText("");watermark.setText("");collate.setChecked(true);for(CheckBox b:new CheckBox[]{reverse,rotate,mirror})b.setChecked(false);}
    JSONObject snapshot()throws JSONException{JSONObject o=new JSONObject();Spinner[] sp={paper,type,quality,color,orientation,nup,parity,margin,sizing};for(int i=0;i<sp.length;i++)o.put("s"+i,sp[i].getSelectedItemPosition());o.put("paper",papers.get(paper.getSelectedItemPosition()).key);o.put("type",types.get(type.getSelectedItemPosition()));o.put("quality",qualities.get(quality.getSelectedItemPosition()));o.put("copies",copies.getText());o.put("scale",scale.getText());o.put("range",range.getText());o.put("watermark",watermark.getText());o.put("collate",collate.isChecked());o.put("reverse",reverse.isChecked());o.put("rotate",rotate.isChecked());o.put("mirror",mirror.isChecked());return o;}
    void savePreset(){if(busy||caps==null)return;EditText name=input("",false);new AlertDialog.Builder(ui).setTitle("Название предустановки").setView(name).setNegativeButton("Отмена",null).setPositiveButton("Сохранить",(d,w)->{try{read();String key=name.getText().toString().trim();if(key.isEmpty())throw new IllegalArgumentException("Введите название");prefs.edit().putString("preset."+key,snapshot().toString()).apply();}catch(Exception e){error(e);}}).show();}
    void loadPreset(){if(busy||caps==null)return;List<String> names=new ArrayList<>();for(String key:prefs.getAll().keySet())if(key.startsWith("preset."))names.add(key.substring(7));Collections.sort(names);if(names.isEmpty()){error(new IOException("Сначала сохраните предустановку"));return;}
        new AlertDialog.Builder(ui).setTitle("Мои предустановки").setItems(names.toArray(new String[0]),(d,index)->{try{JSONObject o=new JSONObject(prefs.getString("preset."+names.get(index),"{}"));Spinner[] sp={paper,type,quality,color,orientation,nup,parity,margin,sizing};for(int i=3;i<sp.length;i++)sp[i].setSelection(Math.max(0,Math.min(sp[i].getCount()-1,o.optInt("s"+i,0))));
            int pi=-1;for(int i=0;i<papers.size();i++)if(papers.get(i).key.equals(o.getString("paper")))pi=i;int ti=types.indexOf(o.getString("type")),qi=qualities.indexOf(o.getInt("quality"));if(pi<0||ti<0||qi<0)throw new IOException("Принтер не поддерживает параметры этой предустановки");paper.setSelection(pi);type.setSelection(ti);quality.setSelection(qi);
            copies.setText(o.getString("copies"));scale.setText(o.getString("scale"));range.setText(o.getString("range"));watermark.setText(o.getString("watermark"));collate.setChecked(o.getBoolean("collate"));reverse.setChecked(o.getBoolean("reverse"));rotate.setChecked(o.getBoolean("rotate"));mirror.setChecked(o.getBoolean("mirror"));
        }catch(Exception e){error(e);}}).setNegativeButton("Закрыть",null).show();
    }
}
