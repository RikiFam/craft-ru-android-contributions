# filmcraft: focused contribution review

One application per branch. Translation, Android and printer work remain separate
directories. This is a review slice of verified bundle `74836d4de05e09f5b2d6921c2d0741f6e6b6eb3a`, not an upstream PR
branch and not a standalone build checkout. No other editor's application patches
or suite-wide ZIPs are included. Shared implementation is linked below at immutable
commits, rather than copied into every branch. Parent history still contains the bundle.

## Review and reproduce

- [Application upstream](https://github.com/storytold/filmcraft); exact base and patch hashes: sources.json.
- [Russian translation and optional spelling instructions](https://github.com/RikiFam/craft-ru-android-contributions/tree/74836d4de05e09f5b2d6921c2d0741f6e6b6eb3a/russifier).
- [Android build instructions and shared platform](https://github.com/RikiFam/craft-ru-android-contributions/tree/74836d4de05e09f5b2d6921c2d0741f6e6b6eb3a/android).
- [Shared spelling sources, templates and tests](https://github.com/RikiFam/craft-ru-android-contributions/tree/74836d4de05e09f5b2d6921c2d0741f6e6b6eb3a/russifier/scripts/artcraft-spelling).
- [Shared dictionaries, original licenses and hashes](https://github.com/RikiFam/craft-ru-android-contributions/tree/74836d4de05e09f5b2d6921c2d0741f6e6b6eb3a/russifier/scripts/artcraft-russian/dictionaries).

Use the complete pinned packages linked above for builds. Apply their optional overlays
in the documented order; the patches here are alternative review views, not a series
to apply one after another. Android patches already include translation prerequisites.
Shared changes affecting several applications must be reviewed in the linked modules too.

## Development and verification

This is personal experimental work made with substantial AI assistance ("vibe coding").
Changes are developed iteratively on Windows and an Android tablet, tested where
possible, committed, then exported as source-only snapshots with documented limitations.
AI-generated code is not presented as professionally audited or production-ready.

The source chat records builds/installations and selected workflows. For spelling,
five existing checkers support native replacement; eight additional panels copy a
corrected fragment for manual insertion. The final Android sweep and broad CI were
incomplete. Publication checks covered source preparation on exact upstream trees,
three shared Rust tests and Android Java/DEX compilation, not a full independent rebuild.
See each package's detailed dated verification notes before relying on a feature.
No APKs or signing keys are distributed. Original copyright and asset/brand restrictions
remain in force; this is not an official Storytold release.

No new direct-printer integration is claimed for this application.

## Android touchpad and DeX overlay: e706b13

This branch includes only filmcraft's updated host files. The common SDK/JNI input
backend, overlay hash guards and build instructions are pinned at
[d78e5df11394a50c5082572aa0488d8cd4815dc0](https://github.com/RikiFam/craft-ru-android-contributions/tree/d78e5df11394a50c5082572aa0488d8cd4815dc0/android).
Use that complete package and its printing, spelling, then DeX overlay sequence
for reproduction. This branch is still a review slice, not a build checkout.
No combined editor archives are copied here.

The source development session checked launches and native DeX captions in all
14 applications. Manual touchpad gesture confirmation is limited to WordCraft.
The publication check compiled the Java/DEX bridge and SDK tests, prepared the
exact pinned winit source, and verified the sequential overlay hashes. It did
not independently rebuild every Rust application or run new device tests.
Experimental AI-assisted (vibe coding) sources only; no APK or signing key.

## Shared DeX pointer-origin correction: e8eb893

Use the updated [complete Android source package](https://github.com/RikiFam/craft-ru-android-contributions/tree/27ed3a01bcdd1cf5ec389a5047dcb5852484d385/android)
and apply its pointer overlay after printing, spelling and DeX. Rebuild the native
host fully. The shared change translates window-relative input to the live child
surface origin; scroll distances and pinch factors are unchanged. No application
host files changed in this update, so this branch links the common implementation.

The source chat confirmed mouse hit-testing in the launcher, installed the fix in
all 14 apps and checked their startup and DeX captions. Individual mouse hit-tests
in every editor were not performed. Publication independently checked overlay hashes,
pinned winit generation and idempotence, not a full Rust rebuild or new device test.
Experimental AI-assisted sources only; no APKs or signing keys.
