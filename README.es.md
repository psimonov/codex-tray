# Codex Tray

[English](README.md) · Español · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Un indicador nativo para la bandeja del sistema de Windows que muestra la cuota restante de Codex.

## Descripción general

Codex Tray permite consultar la cuota actual de Codex sin mantener la aplicación ni la CLI en primer plano. Se ejecuta como una pequeña aplicación en la bandeja del sistema de Windows, reutiliza la sesión autenticada de Codex CLI del usuario actual y muestra un panel compacto al pasar el puntero por el icono.

La aplicación solo se comunica con `codex app-server`, instalado localmente. No solicita una clave de API ni lee o copia directamente `~/.codex/auth.json`.

## Funciones

- Actualizaciones de cuota en tiempo real mediante notificaciones `account/rateLimits/updated`, con una comprobación al pasar el puntero si los datos están desactualizados.
- Panel compacto compatible con DPI y filas estables con el formato `Etiqueta: valor`.
- Compatibilidad con los temas claro y oscuro, el color de énfasis y la transparencia de Windows.
- Iconos alineados a píxeles para niveles de cuota y estados de error.
- El panel aparece en el monitor que contiene el icono al pasar el puntero y se oculta al retirarlo.
- Traducciones integradas a 12 idiomas, con el idioma del sistema seleccionado de forma predeterminada.
- Menú contextual con actualización bajo demanda, acceso a la carpeta del ejecutable, control del inicio con Windows y una acción explícita para cerrar.
- Ajustes portátiles de idioma e inicio guardados junto al ejecutable.
- Sin información sobre herramientas del sistema sobre el icono.
- Estados diferenciados de carga, reconexión, autenticación, suscripción, CLI ausente, cuota agotada y error de app-server.

## Requisitos

- Windows 11 x86-64.
- [Codex CLI](https://learn.chatgpt.com/docs/codex/cli) disponible en `PATH`.
- Una sesión autenticada de Codex CLI creada con `codex login`.

Actualmente, Codex Tray solo implementa un backend nativo para Windows. No se publican artefactos para Linux, macOS ni Windows ARM64 hasta que existan backends implementados y probados para esas plataformas.

## Instalación

1. Abra la [versión más reciente de GitHub](https://github.com/psimonov/codex-tray/releases/latest).
2. Descargue `codex-tray-<version>-windows-x86_64.exe` y su archivo `.sha256`.
3. Verifique la suma SHA-256.
4. Mueva el ejecutable a cualquier carpeta permanente con permisos de escritura y ejecútelo.

Ejemplo de verificación en PowerShell:

```powershell
Get-FileHash .\codex-tray-0.4.1-windows-x86_64.exe -Algorithm SHA256
```

No se requiere instalador. La versión es un único ejecutable portátil; el comando `codex` sigue siendo un requisito externo en tiempo de ejecución.

## Inicio rápido

```powershell
codex login
.\codex-tray-0.4.1-windows-x86_64.exe
```

La aplicación se inicia oculta y añade su icono a la bandeja del sistema de Windows.

## Uso

- Pase el puntero sobre el icono para mostrar el panel de cuota en el mismo monitor.
- Retire el puntero del icono para ocultar el panel.
- Haga clic con el botón derecho para ocultar el panel y abrir el menú contextual.
- Abra **Idioma** y elija **Idioma del sistema** o un idioma concreto. El cambio se aplica de inmediato.
- Seleccione **Actualizar ahora** para repetir inmediatamente `account/read` y `account/rateLimits/read` mediante la conexión existente con app-server.
- Seleccione **Abrir carpeta de la aplicación** para abrir el directorio que contiene el ejecutable en uso.
- Active **Iniciar con Windows** para registrar o quitar la ruta del ejecutable actual en la clave `Run` del usuario.
- Seleccione **Cerrar** para detener Codex Tray y su proceso secundario app-server.

Las actualizaciones llegan mediante una conexión persistente con `codex app-server`. Codex Tray realiza una lectura inicial de la cuenta y los límites, conserva y combina recursivamente las notificaciones parciales posteriores y vuelve a conectarse si app-server termina de forma inesperada. Una actualización explícita repite ambas lecturas. Si el panel se muestra con una instantánea de al menos 30 segundos, Codex Tray la coteja una vez mediante `account/rateLimits/read`; no realiza consultas periódicas en segundo plano.

## Configuración

En el primer inicio, Codex Tray crea `codex-tray.json` junto al ejecutable. El archivo guarda el idioma elegido y la preferencia de inicio con Windows:

```json
{
  "language": "system",
  "start_with_windows": false
}
```

`language` acepta `system`, `en`, `es`, `fr`, `pt`, `de`, `it`, `ru`, `zh-CN`, `hi`, `ar`, `ja` o `ko`. El archivo de configuración es la fuente de verdad. La entrada `Run` del usuario de Windows se sincroniza desde `start_with_windows` y siempre usa la ruta detectada dinámicamente del ejecutable en uso. Si ya existe una entrada de inicio, se importa al crear por primera vez la configuración.

## Compilación desde el código fuente

El repositorio fija la cadena de herramientas de Rust requerida.

```powershell
git clone https://github.com/psimonov/codex-tray.git
Set-Location codex-tray
cargo build --release --locked
```

El ejecutable resultante es `target\release\codex-tray.exe`.

## Pruebas

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## Versiones

Las etiquetas usan el formato `vMAJOR.MINOR.PATCH`. GitHub Actions comprueba que la etiqueta coincida con `Cargo.toml`, ejecuta las verificaciones, compila el ejecutable para Windows x86-64 y publica el archivo junto con su suma SHA-256 en una única versión de GitHub.

El proyecto solo admite Windows en la actualidad, por lo que únicamente se publican artefactos para Windows x86-64. Es una decisión explícita de plataforma, no una afirmación de compatibilidad multiplataforma sin verificar.

## Seguridad

Consulte [SECURITY.md](SECURITY.md) para conocer las versiones compatibles y el canal privado de notificación de vulnerabilidades. No publique vulnerabilidades en incidencias públicas.

## Contribuciones

Consulte [CONTRIBUTING.md](CONTRIBUTING.md) para conocer el flujo de desarrollo y los requisitos de los commits.

## Licencia

Codex Tray está disponible bajo la [licencia MIT](LICENSE).

## Referencia del protocolo

- [Codex App Server](https://learn.chatgpt.com/docs/app-server)
