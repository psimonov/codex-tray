# Codex Tray

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · Italiano · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Un indicatore nativo nell’area di notifica di Windows per monitorare la quota Codex rimanente.

## Panoramica

Codex Tray mantiene visibile la quota Codex corrente senza lasciare l’applicazione o la CLI in primo piano. Funziona come una piccola applicazione nell’area di notifica di Windows, riutilizza la sessione autenticata di Codex CLI dell’utente corrente e mostra un pannello compatto quando il puntatore passa sull’icona.

L’applicazione comunica esclusivamente con `codex app-server`, installato localmente. Non richiede una chiave API e non legge né copia direttamente `~/.codex/auth.json`.

## Funzionalità

- Aggiornamenti in tempo reale della quota tramite notifiche `account/rateLimits/updated`.
- Pannello compatto compatibile con DPI, con righe stabili nel formato `Etichetta: valore`.
- Supporto per tema chiaro e scuro, colore principale e trasparenza di Windows.
- Icone allineate ai pixel per i livelli di quota e gli stati di errore.
- Visualizzazione del pannello al passaggio del puntatore e chiusura quando si allontana.
- Traduzioni integrate in 12 lingue, con la lingua di sistema selezionata per impostazione predefinita.
- Menu contestuale con aggiornamento su richiesta, accesso alla cartella dell’eseguibile, gestione dell’avvio con Windows e un comando esplicito di chiusura.
- Impostazioni portatili di lingua e avvio archiviate accanto all’eseguibile.
- Nessuna descrizione comando di sistema sopra l’icona.
- Stati distinti per caricamento, riconnessione, autenticazione, abbonamento, CLI mancante, quota esaurita ed errore di app-server.

## Requisiti

- Windows 11 x86-64.
- [Codex CLI](https://learn.chatgpt.com/docs/codex/cli) disponibile nel `PATH`.
- Una sessione autenticata di Codex CLI creata con `codex login`.

Codex Tray implementa attualmente solo un backend Windows nativo. Gli artefatti per Linux, macOS e Windows ARM64 non vengono pubblicati finché i relativi backend non saranno implementati e testati.

## Installazione

1. Apri l’[ultima versione su GitHub](https://github.com/psimonov/codex-tray/releases/latest).
2. Scarica `codex-tray-<version>-windows-x86_64.exe` e il relativo file `.sha256`.
3. Verifica il checksum SHA-256.
4. Sposta l’eseguibile in una cartella permanente con permessi di scrittura e avvialo.

Esempio di verifica in PowerShell:

```powershell
Get-FileHash .\codex-tray-0.4.0-windows-x86_64.exe -Algorithm SHA256
```

Non è necessario un programma di installazione. La versione è composta da un singolo eseguibile portabile; il comando `codex` resta un requisito di runtime esterno.

## Avvio rapido

```powershell
codex login
.\codex-tray-0.4.0-windows-x86_64.exe
```

L’applicazione si avvia nascosta e aggiunge la propria icona all’area di notifica di Windows.

## Utilizzo

- Passa il puntatore sull’icona per mostrare il pannello della quota.
- Allontana il puntatore dall’icona per nascondere il pannello.
- Fai clic con il pulsante destro per nascondere il pannello e aprire il menu contestuale.
- Apri **Lingua** e scegli **Lingua di sistema** o una lingua specifica. La modifica viene applicata immediatamente.
- Seleziona **Aggiorna ora** per ripetere immediatamente `account/read` e `account/rateLimits/read` tramite la connessione app-server esistente.
- Seleziona **Apri cartella dell’applicazione** per aprire la directory che contiene l’eseguibile in uso.
- Attiva **Avvia con Windows** per registrare o rimuovere il percorso dell’eseguibile corrente nella chiave `Run` dell’utente.
- Seleziona **Chiudi** per arrestare Codex Tray e il processo figlio app-server.

Gli aggiornamenti arrivano tramite una connessione persistente a `codex app-server`. Codex Tray legge inizialmente l’account e i limiti, ripete entrambe le letture solo dopo un aggiornamento esplicito, unisce le successive notifiche parziali e si riconnette dopo un arresto imprevisto di app-server.

## Configurazione

Al primo avvio, Codex Tray crea `codex-tray.json` accanto all’eseguibile. Il file memorizza la lingua selezionata e la preferenza di avvio con Windows:

```json
{
  "language": "system",
  "start_with_windows": false
}
```

`language` accetta `system`, `en`, `es`, `fr`, `pt`, `de`, `it`, `ru`, `zh-CN`, `hi`, `ar`, `ja` o `ko`. Il file di configurazione è la fonte autorevole delle impostazioni. La voce Windows `Run` dell’utente viene sincronizzata da `start_with_windows` e usa sempre il percorso rilevato dinamicamente dell’eseguibile in esecuzione. Una voce di avvio esistente viene importata alla prima creazione della configurazione.

## Compilazione dal codice sorgente

Il repository fissa la toolchain Rust richiesta.

```powershell
git clone https://github.com/psimonov/codex-tray.git
Set-Location codex-tray
cargo build --release --locked
```

L’eseguibile risultante è `target\release\codex-tray.exe`.

## Test

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## Versioni

I tag usano il formato `vMAJOR.MINOR.PATCH`. GitHub Actions verifica la corrispondenza con `Cargo.toml`, esegue i controlli, compila l’eseguibile Windows x86-64 e pubblica il file insieme al checksum SHA-256 in un’unica versione GitHub.

Il progetto supporta attualmente solo Windows, quindi vengono pubblicati esclusivamente artefatti Windows x86-64. Si tratta di una decisione esplicita sulla piattaforma, non di un’affermazione non verificata di supporto multipiattaforma.

## Sicurezza

Consulta [SECURITY.md](SECURITY.md) per le versioni supportate e il canale privato per segnalare vulnerabilità. Non divulgare vulnerabilità nelle issue pubbliche.

## Contributi

Consulta [CONTRIBUTING.md](CONTRIBUTING.md) per il flusso di sviluppo e i requisiti dei commit.

## Licenza

Codex Tray è disponibile con [licenza MIT](LICENSE).

## Riferimento del protocollo

- [Codex App Server](https://learn.chatgpt.com/docs/app-server)
