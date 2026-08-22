# Codex Tray

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · 日本語 · [한국어](README.ko.md)

---

Codex の残り使用枠を確認するためのネイティブ Windows システムトレイインジケーターです。

## 概要

Codex Tray を使うと、Codex アプリや CLI を前面に表示したままにせず、現在の Codex 使用枠を確認できます。小さな Windows システムトレイアプリとして動作し、現在のユーザーが認証済みの Codex CLI セッションを再利用して、アイコンにポインターを重ねるとコンパクトな使用枠パネルを表示します。

アプリが通信するのは、ローカルにインストールされた `codex app-server` だけです。API キーを要求せず、`~/.codex/auth.json` を直接読み取ったりコピーしたりしません。

## 機能

- `account/rateLimits/updated` サーバー通知による使用枠のリアルタイム更新。
- 安定した `ラベル: 値` 行を使用する、DPI 対応のコンパクトなパネル。
- Windows のライト／ダークテーマ、アクセントカラー、透明効果に対応。
- 使用枠レベルとエラー状態を示すピクセル単位で調整されたトレイアイコン。
- ポインターを重ねるとパネルを表示し、離すと非表示。
- 12 言語の翻訳を内蔵し、既定ではシステム言語を選択。
- オンデマンド更新、実行ファイルのフォルダー表示、Windows と同時に起動する設定、明示的な終了操作を備えたコンテキストメニュー。
- 実行ファイルの隣に保存されるポータブルな言語および自動起動設定。
- トレイアイコン上にシステムツールチップを表示しません。
- 読み込み、再接続、認証、サブスクリプション、CLI 不在、使用枠の枯渇、app-server エラーを個別に表示。

## 必要条件

- Windows 11 x86-64。
- `PATH` から利用できる [Codex CLI](https://learn.chatgpt.com/docs/codex/cli)。
- `codex login` で作成した認証済み Codex CLI セッション。

Codex Tray が現在実装しているのは、ネイティブ Windows backend のみです。Linux、macOS、Windows ARM64 向けの platform backend が実装およびテストされるまで、それらの成果物は公開しません。

## インストール

1. [最新の GitHub Release](https://github.com/psimonov/codex-tray/releases/latest) を開きます。
2. `codex-tray-<version>-windows-x86_64.exe` と対応する `.sha256` ファイルをダウンロードします。
3. SHA-256 チェックサムを検証します。
4. 実行ファイルを書き込み可能な任意の常設フォルダーに移動して実行します。

PowerShell での検証例：

```powershell
Get-FileHash .\codex-tray-0.4.0-windows-x86_64.exe -Algorithm SHA256
```

インストーラーは不要です。Release は単一のポータブル実行ファイルで構成され、`codex` コマンドは引き続き外部の実行要件です。

## クイックスタート

```powershell
codex login
.\codex-tray-0.4.0-windows-x86_64.exe
```

アプリは非表示の状態で起動し、Windows システムトレイにアイコンを追加します。

## 使い方

- 使用枠パネルを表示するには、トレイアイコンにポインターを重ねます。
- パネルを隠すには、アイコンからポインターを離します。
- 右クリックすると、パネルを隠してコンテキストメニューを開きます。
- **言語**サブメニューを開き、**システム言語**または特定の言語を選びます。変更はすぐに反映されます。
- **今すぐ更新**を選ぶと、既存の app-server 接続を通じて `account/read` と `account/rateLimits/read` を直ちに再実行します。
- **アプリケーションフォルダーを開く**を選ぶと、実行中のファイルがあるフォルダーを開きます。
- **Windows と同時に起動**を切り替えると、現在の実行ファイルのパスをユーザーの `Run` キーに登録または削除できます。
- **終了**を選ぶと、Codex Tray とその app-server 子プロセスを停止します。

使用枠の更新は、`codex app-server` への単一の永続接続を通じて届きます。Codex Tray は接続時にアカウントと制限を一度読み取り、明示的な更新時に限って両方を再取得し、その後の部分的な通知をマージして、app-server が予期せず終了した場合は再接続します。

## 設定

初回起動時、Codex Tray は実行ファイルの隣に `codex-tray.json` を作成します。このファイルには、選択した言語と Windows 自動起動設定が保存されます。

```json
{
  "language": "system",
  "start_with_windows": false
}
```

`language` には `system`、`en`、`es`、`fr`、`pt`、`de`、`it`、`ru`、`zh-CN`、`hi`、`ar`、`ja`、`ko` を指定できます。設定ファイルが設定の正本です。ユーザーの Windows `Run` エントリは `start_with_windows` から同期され、常に実行中のファイルから動的に検出したパスを使用します。設定を初めて作成するときは、既存の自動起動エントリが取り込まれます。

## ソースからのビルド

リポジトリでは必要な Rust toolchain を固定しています。

```powershell
git clone https://github.com/psimonov/codex-tray.git
Set-Location codex-tray
cargo build --release --locked
```

生成される実行ファイルは `target\release\codex-tray.exe` です。

## テスト

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## リリース

バージョンタグは `vMAJOR.MINOR.PATCH` 形式です。GitHub Actions はタグと `Cargo.toml` のバージョンが一致することを確認し、プロジェクトのチェックを実行して Windows x86-64 実行ファイルをビルドし、そのファイルと SHA-256 チェックサムを 1 つの GitHub Release に公開します。

現在の対応 OS は Windows のみであるため、公開する成果物も Windows x86-64 のみです。これは明示的なプラットフォーム判断であり、未検証のクロスプラットフォーム対応を主張するものではありません。

## セキュリティ

対応バージョンと非公開の脆弱性報告方法については [SECURITY.md](SECURITY.md) を参照してください。公開 issue で脆弱性を開示しないでください。

## コントリビューション

開発フローと commit の要件については [CONTRIBUTING.md](CONTRIBUTING.md) を参照してください。

## ライセンス

Codex Tray は [MIT License](LICENSE) の下で提供されています。

## プロトコルリファレンス

- [Codex App Server](https://learn.chatgpt.com/docs/app-server)
