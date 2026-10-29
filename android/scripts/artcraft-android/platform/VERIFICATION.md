# Android files and printing - verification, 2026-10-29

Device: Samsung Galaxy Tab S7+ SM-T970, Android 13.
All 13 preview4 APKs installed over existing apps with the same signing key/package IDs.
No app data cleared. Original icons retained. Windows binaries unchanged in this step.
APK sizes and SHA256: apk-builds-2026-10-29.json.

## Printing: verified on tablet

| Editor | Native Android print dialog | Test document |
| --- | --- | --- |
| WordCraft | Passed | One-page test text |
| GridCraft | Passed | Worksheet with ArtCraft-test in A1 |
| DeckCraft | Passed | Blank test presentation |
| CADCraft | Passed | Blank drawing through Plot |
| DesignCraft | Passed | New A4 document |
| PhotoCraft | Passed | Own prior brush-stroke test PSD |
| VectorCraft | Passed | Blank A4 artboard |
| PdfCraft | Passed | PDF saved from GridCraft print preview |
| LightCraft | Passed | Own PNG screenshot imported through Android picker |

These checks establish native dialog entry, not physical output from each editor.
FilmCraft, EffectCraft, FigCraft and SoundCraft have no existing print command found
in the integrated versions; no new paper representation was invented for them.

Installed official Epson Print Enabler 1.5.0. Select
L3250 Series (<printer-address>), the Epson plugin entry, rather than the duplicate
EPSON L3250 Series entry from the built-in service (which initially reports not ready).
The Epson entry reached ready state and rendered a one-page preview.
Expanded Android options show copies, paper, color, orientation, duplex and page range.
Other options opens Epson settings: Print Quality -> Draft / Standard / Best
(displayed in Russian), and layout with borders. Some vendor labels remain English.
The L3250 has manual duplex only: automatic two-sided printing is unavailable in this service.
Do not claim a manual-duplex wizard was implemented. Odd/even page selection in an editor
can be used for two passes with physical paper reversal.

Exactly ONE physical test page was submitted earlier from WordCraft, one copy.
Android spooler recorded completed (state 5, page count 1). User is away, so actual
paper output is not visually confirmed. All later tests used preview/cancel or Save as PDF.
No router changes were needed; printer responded at <printer-address>.

## File checks completed

- WordCraft native Save created Download/ArtCraft-Android-check-20261029.docx.
  Reopened through native Open, appended text, saved to same provider URI.
  Pulled DOCX XML verified exact updated text; 6596 bytes.
- VectorCraft native Save created ArtCraft-vector-check-20261029.vectorcraft, 22387 bytes.
- DeckCraft engine atomic save initially bypassed provider publication (zero-byte file).
  Fixed commit hook, rebuilt and verified ArtCraft-deck-fixed-20261029.deckcraft, 4593 bytes.
- GridCraft Android Save as PDF created artcraft-print-1793302151543953768.PDF, 31218 bytes.
  PdfCraft subsequently opened it through the Android file picker and previewed it for printing.
- LightCraft native import selected an own PNG screenshot. Added missing with_fs loaders;
  rebuilt, reinstalled, verified thumbnail and image in print layout.

## Reproducibility and checks

The shared bridge's 3 focused Rust tests and cargo clippy with warnings denied passed.
All final APKs compiled/packaged and installed successfully. Source patch reverse checks
passed against current source trees. Evidence screenshots are local in tmp/artcraft/android/platform-*.
source-patches/manifest.json records exact Russian-update base commits and binary-safe patches.
Use either these patches OR the idempotent wire scripts on those bases, not both.
Hosts, Java/Rust bridge and build scripts are tracked under scripts/artcraft-android.
Do not regenerate the existing signing key when updating installed apps.

## Limits / remaining broader file validation

Printing renders PDF pages at up to 300 DPI (max bitmap dimension 4096), not vector-preserving.
System printer quality options are available but do not remove this source-rendering limit.
Editor print-layout settings may remain separate from the Android printer options.
Not every editor label is translated; vendor UI is provided by Epson.

The broader all-files request is not fully verified: legacy direct-panel dialog callbacks,
non-primary import/export commands, project sidecars, external media references, folder
outputs, cancellation and provider failures still need coverage. In particular, synchronous
command replay outside a captured command can pick the wrong fallback command; do not
claim every file operation is complete based only on primary Open/Save tests.


## WordCraft input follow-up, 2026-10-29

User confirmed that the previously submitted physical print page came out. No more
physical pages were submitted during input testing. Dictionary work is deferred.

WordCraft now uses a workspace-local patched pinned winit (Cargo cache untouched):
Android ACTION_SCROLL maps to cursor + mouse wheel events; ACTION_MULTIPLE is returned
to Java, whose Unicode characters are drained by the WordCraft host. Soft-input hiding
also handles explicitly shown keyboards. The document requests soft input only after
page taps, not toolbar/background taps. Build-suite invokes the preparation scripts.

On SM-T970: Samsung Russian onscreen keys entered `привет`; ASCII key events continued
inserting after Home/resume in the same process. File menu hid the IME, toolbar taps
left mInputShown=false. ACTION_SCROLL -8 moved the document and +8 returned it.
The Java shell-only ScrollProbe helper reproduces that scroll event; actual physical
two-finger touchpad use still requires user confirmation. Evidence: local
input-resume.png, input-menu.png, input-scroll-new.png.

An earlier captured ANR was input-dispatch timeout with a pending activity relaunch.
Manifests now handle keyboard/navigation configuration changes, avoiding that relaunch
for keyboard changes. This is a mitigation, not proof that every ANR cause is fixed.
Only WordCraft was rebuilt and installed in this follow-up; other manifest edits take
effect at their next build. Full Windows/Android Word-style printer settings UI remains
pending. No dictionary or Windows binary was changed in this input follow-up.

Final APK input7 was reinstalled and verified: Cyrillic `пр` entered; grey background tap hid IME. Restored show_ime_with_hard_keyboard to its original 0. Shared Rust tests: 3 passed; patch reverse-check and preparation idempotence passed.
