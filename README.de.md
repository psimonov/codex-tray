# Codex Tray

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · Deutsch · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Eine native Windows-Taskleistenanzeige für das verbleibende Codex-Nutzungskontingent.

## Überblick

Codex Tray zeigt das aktuelle Codex-Kontingent an, ohne dass die Codex-App oder CLI im Vordergrund bleiben muss. Die kleine Anwendung läuft im Windows-Infobereich, verwendet die authentifizierte Codex-CLI-Sitzung des aktuellen Benutzers und blendet beim Überfahren des Symbols ein kompaktes Kontingentfenster ein.

Die Anwendung kommuniziert ausschließlich mit dem lokal installierten `codex app-server`. Sie fordert keinen API-Schlüssel an und liest oder kopiert `~/.codex/auth.json` nicht direkt.

## Funktionen

- Live-Aktualisierung des Kontingents über `account/rateLimits/updated`-Benachrichtigungen.
- Kompaktes DPI-fähiges Fenster mit stabilen Zeilen im Format `Bezeichnung: Wert`.
- Unterstützung für helles und dunkles Windows-Design, Akzentfarbe und Transparenz.
- Pixelgenau ausgerichtete Symbole für Kontingentstufen und Fehlerzustände.
- Einblenden beim Überfahren des Symbols und Ausblenden, sobald der Zeiger es verlässt.
- Kontextmenü mit Steuerung des Windows-Autostarts und einem eindeutigen Befehl zum Beenden.
- Kein System-Tooltip über dem Symbol.
- Separate Zustände für Laden, erneute Verbindung, Authentifizierung, Abonnement, fehlende CLI, ausgeschöpftes Kontingent und app-server-Fehler.

## Voraussetzungen

- Windows 11 x86-64.
- [Codex CLI](https://learn.chatgpt.com/docs/codex/cli) im `PATH`.
- Eine mit `codex login` erstellte, authentifizierte Codex-CLI-Sitzung.

Codex Tray besitzt derzeit ausschließlich ein natives Windows-Backend. Artefakte für Linux, macOS und Windows ARM64 werden erst veröffentlicht, wenn die entsprechenden Plattform-Backends implementiert und getestet sind.

## Installation

1. Öffnen Sie das [neueste GitHub Release](https://github.com/psimonov/codex-tray/releases/latest).
2. Laden Sie `codex-tray-<version>-windows-x86_64.exe` und die zugehörige `.sha256`-Datei herunter.
3. Prüfen Sie die SHA-256-Prüfsumme.
4. Verschieben Sie die ausführbare Datei in einen dauerhaften Ordner und starten Sie sie.

Beispiel für die Prüfung in PowerShell:

```powershell
Get-FileHash .\codex-tray-0.2.0-windows-x86_64.exe -Algorithm SHA256
```

Ein Installationsprogramm ist nicht erforderlich. Das Release besteht aus einer einzelnen portablen ausführbaren Datei; der Befehl `codex` bleibt eine externe Laufzeitvoraussetzung.

## Schnellstart

```powershell
codex login
.\codex-tray-0.2.0-windows-x86_64.exe
```

Die Anwendung startet ausgeblendet und fügt ihr Symbol dem Windows-Infobereich hinzu.

## Verwendung

- Fahren Sie mit dem Zeiger über das Symbol, um das Kontingentfenster anzuzeigen.
- Bewegen Sie den Zeiger vom Symbol weg, um das Fenster auszublenden.
- Klicken Sie mit der rechten Maustaste, um das Fenster auszublenden und das Kontextmenü zu öffnen.
- Schalten Sie **Mit Windows starten** um, um den aktuellen Pfad der ausführbaren Datei im benutzerspezifischen `Run`-Schlüssel einzutragen oder zu entfernen.
- Wählen Sie **Beenden**, um Codex Tray und seinen app-server-Kindprozess zu stoppen.

Aktualisierungen treffen über eine dauerhafte Verbindung zu `codex app-server` ein. Codex Tray liest Konto und Limits anfangs einmalig, führt spätere unvollständige Benachrichtigungen zusammen und verbindet sich nach einem unerwarteten Ende von app-server erneut.

## Konfiguration

Codex Tray verwendet weder eine Konfigurationsdatei noch Umgebungsvariablen. Der optionale Windows-Autostart wird im Kontextmenü gesteuert und verwendet immer den dynamisch ermittelten Pfad der laufenden ausführbaren Datei.

## Aus dem Quellcode erstellen

Das Repository legt die benötigte Rust-Toolchain fest.

```powershell
git clone https://github.com/psimonov/codex-tray.git
Set-Location codex-tray
cargo build --release --locked
```

Die erzeugte ausführbare Datei ist `target\release\codex-tray.exe`.

## Tests

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## Releases

Versionstags verwenden das Format `vMAJOR.MINOR.PATCH`. GitHub Actions gleicht den Tag mit `Cargo.toml` ab, führt alle Prüfungen aus, erstellt die ausführbare Windows-x86-64-Datei und veröffentlicht sie zusammen mit ihrer SHA-256-Prüfsumme in einem GitHub Release.

Das Projekt unterstützt derzeit nur Windows, daher werden ausschließlich Windows-x86-64-Artefakte veröffentlicht. Dies ist eine ausdrückliche Plattformentscheidung und keine ungeprüfte Behauptung plattformübergreifender Unterstützung.

## Sicherheit

Unterstützte Versionen und der private Meldeweg für Sicherheitslücken sind in [SECURITY.md](SECURITY.md) beschrieben. Veröffentlichen Sie Sicherheitslücken nicht in öffentlichen Issues.

## Mitwirken

Der Entwicklungsablauf und die Commit-Anforderungen sind in [CONTRIBUTING.md](CONTRIBUTING.md) beschrieben.

## Lizenz

Codex Tray ist unter der [MIT-Lizenz](LICENSE) verfügbar.

## Protokollreferenz

- [Codex App Server](https://learn.chatgpt.com/docs/app-server)
