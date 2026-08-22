# Codex Tray

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · Português · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Um indicador nativo na bandeja do sistema do Windows para acompanhar a cota restante do Codex.

## Visão geral

O Codex Tray mantém a cota atual do Codex visível sem exigir que o aplicativo ou a CLI permaneçam em primeiro plano. Ele é executado como um pequeno aplicativo na bandeja do sistema do Windows, reutiliza a sessão autenticada do Codex CLI do usuário atual e exibe um painel compacto ao passar o cursor sobre o ícone.

O aplicativo se comunica apenas com o `codex app-server` instalado localmente. Ele não solicita uma chave de API e não lê nem copia diretamente `~/.codex/auth.json`.

## Recursos

- Atualizações de cota em tempo real por meio de notificações `account/rateLimits/updated`, com uma verificação ao passar o cursor quando os dados estão desatualizados.
- Painel compacto com suporte a DPI e linhas estáveis no formato `Rótulo: valor`.
- Suporte aos temas claro e escuro, à cor de destaque e à transparência do Windows.
- Ícones alinhados aos pixels para níveis de cota e estados de erro.
- Exibição do painel no monitor que contém o ícone ao passar o cursor e ocultação ao afastá-lo.
- Traduções integradas para 12 idiomas, com o idioma do sistema selecionado por padrão.
- Menu de contexto com atualização sob demanda, acesso à pasta do executável, controle de inicialização com o Windows e uma ação explícita para fechar.
- Configurações portáteis de idioma e inicialização armazenadas ao lado do executável.
- Nenhuma dica de ferramenta do sistema sobre o ícone.
- Estados distintos para carregamento, reconexão, autenticação, assinatura, CLI ausente, cota esgotada e erro do app-server.

## Requisitos

- Windows 11 x86-64.
- [Codex CLI](https://learn.chatgpt.com/docs/codex/cli) disponível no `PATH`.
- Uma sessão autenticada do Codex CLI criada com `codex login`.

Atualmente, o Codex Tray implementa apenas um backend nativo do Windows. Artefatos para Linux, macOS e Windows ARM64 não são publicados até que esses backends de plataforma sejam implementados e testados.

## Instalação

1. Abra a [versão mais recente no GitHub](https://github.com/psimonov/codex-tray/releases/latest).
2. Baixe `codex-tray-<version>-windows-x86_64.exe` e o arquivo `.sha256` correspondente.
3. Verifique a soma SHA-256.
4. Mova o executável para uma pasta permanente com permissão de escrita e execute-o.

Exemplo de verificação no PowerShell:

```powershell
Get-FileHash .\codex-tray-0.4.1-windows-x86_64.exe -Algorithm SHA256
```

Não é necessário um instalador. A versão é um único executável portátil; o comando `codex` continua sendo um requisito externo de execução.

## Início rápido

```powershell
codex login
.\codex-tray-0.4.1-windows-x86_64.exe
```

O aplicativo inicia oculto e adiciona seu ícone à bandeja do sistema do Windows.

## Uso

- Passe o cursor sobre o ícone para exibir o painel da cota no mesmo monitor.
- Afaste o cursor do ícone para ocultar o painel.
- Clique com o botão direito para ocultar o painel e abrir o menu de contexto.
- Abra **Idioma** e escolha **Idioma do sistema** ou um idioma específico. A alteração é aplicada imediatamente.
- Selecione **Atualizar agora** para repetir imediatamente `account/read` e `account/rateLimits/read` pela conexão existente com o app-server.
- Selecione **Abrir pasta do aplicativo** para abrir o diretório que contém o executável em uso.
- Alterne **Iniciar com o Windows** para registrar ou remover o caminho do executável atual na chave `Run` do usuário.
- Selecione **Fechar** para encerrar o Codex Tray e seu processo filho app-server.

As atualizações chegam por uma conexão persistente com `codex app-server`. O Codex Tray lê inicialmente a conta e os limites, preserva e combina recursivamente as notificações parciais posteriores e se reconecta após uma interrupção inesperada do app-server. Uma atualização explícita repete ambas as leituras. Se o painel for exibido com um instantâneo de pelo menos 30 segundos, o Codex Tray o reconcilia uma vez por meio de `account/rateLimits/read`; não há consulta periódica em segundo plano.

## Configuração

Na primeira inicialização, o Codex Tray cria `codex-tray.json` ao lado do executável. O arquivo armazena o idioma selecionado e a preferência de inicialização com o Windows:

```json
{
  "language": "system",
  "start_with_windows": false
}
```

`language` aceita `system`, `en`, `es`, `fr`, `pt`, `de`, `it`, `ru`, `zh-CN`, `hi`, `ar`, `ja` ou `ko`. O arquivo de configuração é a fonte da verdade. A entrada `Run` do usuário do Windows é sincronizada a partir de `start_with_windows` e sempre usa o caminho detectado dinamicamente do executável em execução. Uma entrada de inicialização existente é importada quando a configuração é criada pela primeira vez.

## Compilação a partir do código-fonte

O repositório fixa a cadeia de ferramentas Rust necessária.

```powershell
git clone https://github.com/psimonov/codex-tray.git
Set-Location codex-tray
cargo build --release --locked
```

O executável resultante é `target\release\codex-tray.exe`.

## Testes

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## Versões

As tags usam o formato `vMAJOR.MINOR.PATCH`. O GitHub Actions verifica a versão em `Cargo.toml`, executa as validações, compila o executável Windows x86-64 e publica o arquivo e sua soma SHA-256 em uma única versão do GitHub.

Hoje o projeto oferece suporte apenas ao Windows, portanto somente artefatos Windows x86-64 são publicados. Essa é uma decisão explícita de plataforma, não uma alegação não verificada de suporte multiplataforma.

## Segurança

Consulte [SECURITY.md](SECURITY.md) para ver as versões com suporte e o canal privado para relatar vulnerabilidades. Não divulgue vulnerabilidades em issues públicas.

## Contribuição

Consulte [CONTRIBUTING.md](CONTRIBUTING.md) para conhecer o fluxo de desenvolvimento e os requisitos de commits.

## Licença

O Codex Tray está disponível sob a [licença MIT](LICENSE).

## Referência do protocolo

- [Codex App Server](https://learn.chatgpt.com/docs/app-server)
