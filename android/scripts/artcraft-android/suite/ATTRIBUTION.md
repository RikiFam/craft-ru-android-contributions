# Значки Android-приложений

Значки берутся без изменения из локальных исходников соответствующего приложения:
`tmp/artcraft/<app>/repo/assets/app-icon/hicolor/512x512/apps/ai.storyteller.<app>.png`.
Источник каждого — официальный репозиторий `https://github.com/storytold/<app>`;
авторы — соответствующие участники проекта ArtCraft. Файлы не добавляются в git
повторно, а копируются в ресурсы APK при сборке.

PhotoCraft: исходное оформление владельца проекта, MIT OR Apache-2.0;
см. `assets/app-icon/LICENSE.txt` исходного репозитория. Остальные изображения
приложений сохраняют атрибуцию и лицензии, записанные в их `ATTRIBUTION.md` /
`ATTRIBUTION.toml` и боковых файлах лицензий.

Знак ArtCraft Launcher берётся из
`tmp/artcraft/craft-launcher/repo/assets/app-icon/hicolor/512x512/apps/ai.storyteller.artcraft-launcher.png`.
Это исходный знак ArtCraft Team, товарный знак, не MIT/Apache;
см. `assets/app-icon/LICENSE.txt` и `docs/brand/LICENSE-brand.txt` в
`https://github.com/storytold/craft-launcher`. Здесь он использован по прямому
запросу пользователя для его локальной установки. Публикация этих APK не выполнялась.
