package ai.storyteller.android;

import android.app.NativeActivity;
import android.content.Intent;
import android.database.Cursor;
import android.net.Uri;
import android.os.Bundle;
import android.provider.DocumentsContract;
import android.provider.OpenableColumns;
import android.webkit.MimeTypeMap;
import java.io.*;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicInteger;
import org.json.*;

/** Android SDK boundary only. Editors and document formats remain native Rust. */
public class CraftActivity extends NativeActivity {
    /** Use Android's real clipboard so a spelling correction is available to the IME. */
    public boolean copyProofText(String text) {
        if (text == null || text.length() > 1000000) return false;
        try {
            android.content.ClipboardManager clipboard =
                (android.content.ClipboardManager) getSystemService(CLIPBOARD_SERVICE);
            if (clipboard == null) return false;
            clipboard.setPrimaryClip(android.content.ClipData.newPlainText("ArtCraft", text));
            return true;
        } catch (RuntimeException error) {
            return false;
        }
    }
    private final java.util.HashMap<Integer, CraftTouchpad> touchpads = new java.util.HashMap<>();
    public synchronized float[] processTouchpad(int device, int action, long down, long time, float[] pointers) {
        CraftTouchpad pad = touchpads.get(device);
        if (pad == null) {
            if (touchpads.size() >= 16) touchpads.clear();
            pad = new CraftTouchpad(this); touchpads.put(device, pad);
        }
        return pad.process(device, action, down, time, pointers);
    }
    private final AtomicInteger ids = new AtomicInteger(4000);
    private final ConcurrentHashMap<Integer, String> answers = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<Integer, String> kinds = new ConcurrentHashMap<>();
    private final ExecutorService io = Executors.newSingleThreadExecutor();
    private File mirrors;
    private final ConcurrentLinkedQueue<String> committedText = new ConcurrentLinkedQueue<>();
    @Override public boolean dispatchKeyEvent(android.view.KeyEvent event) {
        // Native AKeyEvent has no characters payload for ACTION_MULTIPLE. Keep the
        // Unicode text before forwarding ordinary key presses to the native queue.
        if (event.getAction() == android.view.KeyEvent.ACTION_MULTIPLE && event.getCharacters() != null) {
            committedText.add(event.getCharacters());
            return true;
        }
        return super.dispatchKeyEvent(event);
    }
    public String pollCommittedText() {
        JSONArray values = new JSONArray();
        for (String text; (text = committedText.poll()) != null;) values.put(text);
        return values.toString();
    }
    private android.content.SharedPreferences links;

    @Override public void onCreate(Bundle state) {
        mirrors = new File(getFilesDir(), "documents");
        mirrors.mkdirs();
        links = getSharedPreferences("document-uris", MODE_PRIVATE);
        super.onCreate(state);
        // NativeActivity takes the entire Window surface, bypassing Java decor
        // rendering (including the DeX caption). Give native rendering a child
        // SurfaceView while retaining NativeContentView for IME and content bounds.
        android.view.ViewGroup content = findViewById(android.R.id.content);
        android.view.View nativeContent = content.getChildAt(0);
        content.removeView(nativeContent);
        android.widget.FrameLayout frame = new android.widget.FrameLayout(this);
        android.view.SurfaceView surface = new android.view.SurfaceView(this);
        surface.getHolder().addCallback(this);
        frame.addView(surface, new android.widget.FrameLayout.LayoutParams(-1, -1));
        frame.addView(nativeContent, new android.widget.FrameLayout.LayoutParams(-1, -1));
        getWindow().takeSurface(null);
        setContentView(frame);
        nativeContent.requestFocus();
    }

    public int select(String kind, String name) {
        final int id = ids.incrementAndGet();
        kinds.put(id, kind);
        runOnUiThread(() -> {
            try {
                Intent intent = new Intent(kind.equals("save") ? Intent.ACTION_CREATE_DOCUMENT :
                    kind.equals("folder") ? Intent.ACTION_OPEN_DOCUMENT_TREE : Intent.ACTION_OPEN_DOCUMENT);
                intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION |
                    Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION);
                if (!kind.equals("folder")) {
                    intent.addCategory(Intent.CATEGORY_OPENABLE);
                    intent.setType(kind.equals("save") ? mime(name) : "*/*");
                    intent.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, kind.equals("many"));
                    if (kind.equals("save")) intent.putExtra(Intent.EXTRA_TITLE, safeName(new File(name.replace('\\', '/')).getName()));
                }
                startActivityForResult(intent, id);
            } catch (Exception e) { complete(id, null, e); }
        });
        return id;
    }
    public String poll(int id) { return answers.remove(id); }
    @Override protected void onDestroy() {
        for (Integer id : kinds.keySet()) complete(id, null, new IOException("Окно приложения закрыто"));
        io.shutdown();
        super.onDestroy();
    }
    public String printCache() { return getCacheDir().getAbsolutePath(); }
    private void complete(int id, List<String> paths, Exception error) {
        try {
            JSONObject result = new JSONObject();
            if (error != null) result.put("error", "Не удалось выполнить файловую операцию: " + error.getMessage());
            else result.put("paths", new JSONArray(paths == null ? Collections.emptyList() : paths));
            answers.put(id, result.toString());
        } catch (JSONException impossible) { throw new IllegalStateException(impossible); }
        kinds.remove(id);
    }
    @Override protected void onActivityResult(int request, int result, Intent data) {
        super.onActivityResult(request, result, data);
        final String kind = kinds.get(request);
        if (kind == null) return;
        if (result != RESULT_OK || data == null) { complete(request, null, null); return; }
        io.execute(() -> {
            try {
                List<Uri> uris = new ArrayList<>();
                if (data.getClipData() != null) {
                    for (int i=0; i<data.getClipData().getItemCount(); i++) uris.add(data.getClipData().getItemAt(i).getUri());
                } else if (data.getData() != null) uris.add(data.getData());
                List<String> paths = new ArrayList<>();
                for (Uri uri : uris) {
                    int flags = data.getFlags() & (Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION);
                    getContentResolver().takePersistableUriPermission(uri, flags);
                    File dir = new File(mirrors, UUID.randomUUID().toString());
                    if (!dir.mkdirs()) throw new IOException("Не удалось создать рабочую папку");
                    if (kind.equals("folder")) {
                        Uri document = DocumentsContract.buildDocumentUriUsingTree(uri, DocumentsContract.getTreeDocumentId(uri));
                        remember(dir, document);
                        copyTree(document, dir, 0);
                        paths.add(dir.getCanonicalPath());
                    } else {
                        File file = new File(dir, displayName(uri));
                        if (!kind.equals("save")) copyIn(uri, file);
                        // A new save target intentionally stays absent until the editor writes it.
                        remember(file, uri);
                        paths.add(file.getCanonicalPath());
                    }
                }
                complete(request, paths, null);
            } catch (Exception e) { complete(request, null, e); }
        });
    }
    private String displayName(Uri uri) throws IOException {
        try (Cursor c = getContentResolver().query(uri, new String[]{OpenableColumns.DISPLAY_NAME}, null,null,null)) {
            if (c != null && c.moveToFirst()) return safeName(c.getString(0));
        }
        throw new IOException("Хранилище не сообщило имя файла");
    }
    static String safeName(String name) {
        if (name == null || name.isEmpty()) return "Документ";
        name = name.replace('/', '_').replace('\\', '_').replace('\0', '_');
        return name.equals(".") || name.equals("..") ? "Документ" : name;
    }
    static String mime(String name) {
        int dot = name.lastIndexOf('.');
        String type = dot < 0 ? null : MimeTypeMap.getSingleton().getMimeTypeFromExtension(name.substring(dot+1).toLowerCase(Locale.ROOT));
        return type == null ? "application/octet-stream" : type;
    }
    private void remember(File file, Uri uri) throws IOException {
        if (!links.edit().putString(file.getCanonicalPath(), uri.toString()).commit())
            throw new IOException("Не удалось запомнить выбранное хранилище");
    }
    private static void transfer(InputStream in, OutputStream out) throws IOException {
        if (in == null || out == null) throw new IOException("Хранилище не открыло файл");
        byte[] buffer = new byte[65536]; int n;
        while ((n = in.read(buffer)) != -1) out.write(buffer,0,n);
        out.flush();
    }
    private void copyIn(Uri uri, File file) throws IOException {
        try (InputStream in=getContentResolver().openInputStream(uri); OutputStream out=new FileOutputStream(file)) { transfer(in,out); }
    }
    private void copyTree(Uri directory, File local, int depth) throws IOException {
        if (depth > 64) throw new IOException("Слишком много уровней вложенных папок");
        Uri children = DocumentsContract.buildChildDocumentsUriUsingTree(directory, DocumentsContract.getDocumentId(directory));
        String[] columns = {DocumentsContract.Document.COLUMN_DOCUMENT_ID, DocumentsContract.Document.COLUMN_DISPLAY_NAME, DocumentsContract.Document.COLUMN_MIME_TYPE};
        try (Cursor c = getContentResolver().query(children,columns,null,null,null)) {
            if (c == null) throw new IOException("Папка недоступна");
            while (c.moveToNext()) {
                Uri uri = DocumentsContract.buildDocumentUriUsingTree(directory,c.getString(0));
                File target = new File(local,safeName(c.getString(1)));
                if (target.exists()) throw new IOException("Совпадают имена файлов в папке");
                if (DocumentsContract.Document.MIME_TYPE_DIR.equals(c.getString(2))) {
                    if (!target.mkdir()) throw new IOException("Не удалось создать подпапку");
                    copyTree(uri,target,depth+1);
                } else copyIn(uri,target);
                remember(target, uri);
            }
        }
    }
    /** Called after a successful local write. An error must be reported as a failed save. */
    private Uri mappedDirectory(File directory, int depth) throws IOException {
        if (directory == null || depth > 64 || !directory.getCanonicalFile().toPath().startsWith(mirrors.getCanonicalFile().toPath()))
            throw new IOException("Для файла не выбрана папка Android");
        String known = links.getString(directory.getCanonicalPath(), null);
        if (known != null) return Uri.parse(known);
        Uri parent = mappedDirectory(directory.getParentFile(), depth + 1);
        Uri created = DocumentsContract.createDocument(getContentResolver(), parent, DocumentsContract.Document.MIME_TYPE_DIR, directory.getName());
        if (created == null) throw new IOException("Не удалось создать папку Android");
        remember(directory, created);
        return created;
    }
    public synchronized String commitDocument(String path) {
        try {
            File file = new File(path).getCanonicalFile();
            if (!file.toPath().startsWith(mirrors.getCanonicalFile().toPath())) return "";
            String mapped = links.getString(file.getPath(), null);
            if (mapped == null) {
                Uri parent = mappedDirectory(file.getParentFile(), 0);
                Uri created = DocumentsContract.createDocument(getContentResolver(),parent,mime(file.getName()),file.getName());
                if (created == null) throw new IOException("Не удалось создать файл Android");
                mapped = created.toString();
                remember(file, Uri.parse(mapped));
            }
            try (InputStream in=new FileInputStream(file); OutputStream out=getContentResolver().openOutputStream(Uri.parse(mapped),"wt")) { transfer(in,out); }
            return "";
        } catch (Exception e) { return "Файл не сохранён в выбранное хранилище: " + e.getMessage(); }
    }
    public String printDocument(String path, String title) {
        File file = new File(path);
        if (!file.isFile()) return "Не найден документ для печати";
        runOnUiThread(() -> {
            new CraftPrinterDialog(this,file,title).show();
        });
        return "";
    }
}
