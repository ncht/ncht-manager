# NCHT Manager
## 概要
Discordの使われていないチャンネルを整理するbotです。

## 環境変数
- THRESHOLD_DAYS
    指定した日数以上書き込まれていないチャンネルを`ARCHIVE_CATEGORY`に移動する
- ACTIVE_CATEGORY
    整理する対象のカテゴリ名
- ARCHIVE_CATEGORY
    アーカイブ先のカテゴリ名
- DISCORD_TOKEN
    bot用のトークン

## コマンド
- !archive
    - `ACTIVE_CATEGORY`にある使われていないチャンネルを`ARCHIVE_CATEGORY`に移動する
- !restore
    - `ARCHIVE_CATEGORY`にあるチャンネルを`ACTIVE_CATEGORY`に戻す
    - `ARCHIVE_CATEGORY`にあるチャンネルでのみ有効
- !role
    - チャンネル名と同じロールを作成して全メンバーにロールを付与する
