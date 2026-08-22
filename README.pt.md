# Codex Tray

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · Português · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Um indicador nativo na bandeja do sistema do Windows para acompanhar a cota restante do Codex.

## Visão geral

O Codex Tray mantém a cota atual do Codex visível sem exigir que o aplicativo ou a CLI permaneçam em primeiro plano. Ele é executado como um pequeno aplicativo na bandeja do sistema do Windows, reutiliza a sessão autenticada do Codex CLI do usuário atual e exibe um painel compacto ao passar o cursor sobre o ícone.

O aplicativo se comunica apenas com o `codex app-server` instalado localmente. Ele não solicita uma chave de API e não lê nem copia diretamente `~/.codex/auth.json`.

## Recursos

- Atualizações de cota em tempo real por meio de notificações `account/rateLimits/updated`.
- Painel compacto com suporte a DPI e linhas estáveis no formato `Rótulo: valor`.
- Suporte aos temas claro e escuro, à cor de destaque e à transparência do Windows.
- Ícones alinhados aos pixels para níveis de cota e estados de erro.
- Exibição do painel ao passar o cursor e ocultação ao afastá-lo.
- Menu de contexto com controle de inicialização com o Windows e uma ação explícita para fechar.
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
4. Mova o executável para uma pasta permanente e execute-o.

Exemplo de verificação no PowerShell:

```powershell
Get-FileHash .\codex-tray-0.2.0-windows-x86_64.exe -Algorithm SHA256
```

Não é necessário um instalador. A versão é um único executável portátil; o comando `codex` continua sendo um requisito externo de execução.

## Início rápido

```powershell
codex login
.\codex-tray-0.2.0-windows-x86_64.exe
```

O aplicativo inicia oculto e adiciona seu ícone à bandeja do sistema do Windows.

## Uso

- Passe o cursor sobre o ícone para exibir o painel da cota.
- Afaste o cursor do ícone para ocultar o painel.
- Clique com o botão direito para ocultar o painel e abrir o menu de contexto.
- Alterne **Iniciar com o Windows** para registrar ou remover o caminho do executável atual na chave `Run` do usuário.
- Selecione **Fechar** para encerrar o Codex Tray e seu processo filho app-server.

As atualizações chegam por uma conexão persistente com `codex app-server`. O Codex Tray realiza uma leitura inicial da conta e dos limites, combina as notificações parciais posteriores e se reconecta após uma interrupção inesperada do app-server.

## Configuração

O Codex Tray não possui arquivo de configuração nem variáveis de ambiente. A inicialização opcional com o Windows é controlada pelo menu de contexto e sempre usa o caminho detectado dinamicamente do executável em execução.

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
