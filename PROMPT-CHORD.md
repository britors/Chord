# PROMPT DE IMPLEMENTAÇÃO — CHORD (TERMINAL DO LYRA OS)

> **Versão:** 2.0
> **Status:** Especificação de build congelada, pronta para implementação
> **Supersede:** `PROMPT-CORDA.md` v1.0 — projeto renomeado de **Corda** para **Chord**, e arquitetura revisada para núcleo compartilhado entre os frontends GTK4 e Qt (antes só GTK4 estava especificado).
> **Pré-requisitos:** `PROMPT-LYRA-IDENTIDADE.md` v1.0 (paleta, tokens visuais)
> **Escopo:** Emulador de terminal do ecossistema Lyra, com dois frontends nativos (GTK4 para GNOME, Qt para KDE) compartilhando um núcleo Rust único. Todas as decisões abaixo estão fechadas.

---

## 1. Visão Geral e Nome

### 1.1 Por que Chord

- **Chord** (acorde) soa bem tanto em português quanto em inglês — diferente de "Corda", que não se traduz foneticamente bem fora do português
- Evolui a metáfora musical: uma corda sozinha é uma nota; um **acorde** é várias cordas soando juntas — reflete bem um terminal moderno, que existe pra múltiplas abas/painéis trabalharem em conjunto
- Curto, fácil de digitar, memorável

### 1.2 Identidade do projeto

| Item | Valor |
|---|---|
| Nome | Chord |
| Nome do pacote | `lyra-chord` |
| Licença | GPLv3 |
| Idioma | pt-BR (via `gettext`), pronto para outros idiomas — nome do produto já pensado para soar bem também em inglês |

### 1.3 Princípios

1. **Não reinventar a emulação de terminal.** Usar o motor **VTE** — mesma base do GNOME Console, `gnome-terminal`, Tilix. Foco em experiência (UI, tema, atalhos), não em parser de sequências de escape.
2. **Um núcleo, dois rostos.** Toda a lógica que não é renderização de UI vive numa crate Rust compartilhada — GTK4 e Qt são consumidores finos dela, não implementações paralelas divergentes.
3. **Nativo primeiro, em ambos os frontends.** Sem Electron, sem WebView — GTK4 puro de um lado, Qt/QML puro do outro, os dois em Rust.

---

## 2. Arquitetura — Núcleo Compartilhado

### 2.1 Por que compartilhar, e como

Um workspace Cargo com uma crate central (`chord-core`) sem nenhuma dependência de toolkit de UI, consumida por dois crates de frontend finos:

```
chord/
├── Cargo.toml                       # workspace root
├── chord-core/                      # SEM UI — lógica pura
│   ├── src/
│   │   ├── lib.rs
│   │   ├── theme.rs                 # parser de palette.json → tokens de cor/paleta ANSI
│   │   ├── profile.rs                # perfis de shell, fontes, atalhos
│   │   ├── config.rs                 # leitura/escrita de configuração (GSettings-agnóstico — ver §2.3)
│   │   └── i18n.rs                   # catálogo de strings (gettext)
│   └── Cargo.toml
├── chord-gtk/                        # frontend GNOME
│   ├── src/
│   │   ├── main.rs
│   │   ├── window.rs
│   │   ├── terminal_pane.rs          # wrapper sobre o widget VTE (vte4-rs)
│   │   └── tab_bar.rs
│   └── Cargo.toml                    # depende de chord-core + gtk4-rs + vte4
├── chord-qt/                         # frontend KDE
│   ├── src/
│   │   ├── main.rs
│   │   ├── window.rs
│   │   ├── terminal_pane.rs          # wrapper sobre QTermWidget ou VTE via binding Qt
│   │   └── tab_bar.rs
│   └── Cargo.toml                    # depende de chord-core + cxx-qt
├── data/
│   ├── icons/
│   └── themes/
│       └── chord-dark.json           # artefato derivado de palette.json (gerado por chord-core::theme)
├── po/
└── README.md
```

### 2.2 Frontend Qt em Rust puro (via `cxx-qt`)

Decisão central desta versão: o frontend Qt **não é escrito em C++**. Usa **`cxx-qt`** (biblioteca com apoio da comunidade KDE) para escrever QML/Qt em Rust, permitindo que `chord-qt` dependa de `chord-core` como uma crate Rust normal — **sem fronteira de FFI, sem binding manual, sem duplicar lógica**.

Isso evita o problema mais comum de projetos "multi-toolkit": a versão secundária (aqui, Qt) divergir silenciosamente da principal (GTK4) porque cada uma reimplementou a mesma regra à sua maneira. Com `chord-core` compartilhado, um bug corrigido ou uma regra de negócio ajustada corrige os dois frontends ao mesmo tempo, por definição.

### 2.3 Motor de terminal por frontend

- **GTK4:** widget **VTE** via bindings `vte4-rs` (padrão GNOME)
- **Qt:** **QTermWidget** (biblioteca C++ madura, usada pelo Konsole/outros terminais KDE) via binding `cxx` a partir do `chord-qt`, ou alternativa nativa em Rust caso surja opção madura no futuro
- Ambos os motores implementam a mesma paleta ANSI de 16 cores gerada por `chord-core::theme` — a fonte de verdade da cor é sempre a crate compartilhada, nunca hardcoded em cada frontend

### 2.4 Configuração multiplataforma

Como o projeto roda tanto em GNOME (onde GSettings é o padrão natural) quanto em KDE (onde a convenção é `KConfig`), `chord-core::config` define um formato de configuração **próprio e neutro** (arquivo TOML em `~/.config/chord/config.toml`), lido e escrito por ambos os frontends da mesma forma. Isso evita duas fontes de configuração divergentes (uma em dconf, outra em kconfig) para o mesmo app.

---

## 3. Funcionalidades (válidas para ambos os frontends)

### 3.1 Essenciais (v1)

- **Abas** — múltiplas sessões de shell na mesma janela (`Ctrl+Shift+T` nova aba, `Ctrl+Shift+W` fechar, `Ctrl+Tab`/`Ctrl+Shift+Tab` navegar)
- **Divisão de painel** — split horizontal e vertical na mesma aba (`Ctrl+Shift+O` horizontal, `Ctrl+Shift+E` vertical)
- **Cópia/colagem** — `Ctrl+Shift+C`/`Ctrl+Shift+V`
- **Busca no scrollback** — `Ctrl+Shift+F`, com realce de resultados
- **Zoom de fonte** — `Ctrl+scroll` ou `Ctrl+/Ctrl-`
- **Perfis de shell** — detecção automática do shell padrão (`$SHELL`), com opção de sobrescrever por aba/painel

### 3.2 Tema visual

- **Tema padrão "Chord Dark"**, construído a partir dos tokens de `PROMPT-LYRA-IDENTIDADE.md` §1.1:

| Elemento | Token |
|---|---|
| Fundo | `lyra-night` (#16191D), transparência configurável (0-100%, padrão 100% opaco) |
| Texto padrão | `lyra-star` (#E8ECFF) |
| Cursor | `lyra-neon` (#A78BFA), piscante por padrão |
| Seleção | `lyra-sapphire` (#2D5BE3) a 40% de opacidade |
| Paleta ANSI (16 cores) | Gerada por `chord-core::theme` a partir de `palette.json` — ver `data/themes/chord-dark.json` como artefato derivado, nunca hardcoded solto em cada frontend |

- **Temas alternativos:** suporte a formato compatível com os já populares (base16/iTerm) — permite trazer Catppuccin, Nord, Dracula, Tokyo Night sem esforço de conversão manual. O parser de tema externo vive em `chord-core`, então funciona identicamente nos dois frontends
- **Fonte padrão:** `JetBrains Mono` (boa legibilidade, ligaduras opcionais, licença livre)

### 3.3 Fora do v1

- Multiplexação de sessão persistente (tipo tmux embutido)
- Integração com o Vega (ex.: abrir terminal dentro de snapshot montado)
- Sincronização de perfil/tema entre máquinas

---

## 4. Empacotamento

Seguindo o padrão multi-distro já validado com Vega e Lyra Tour:

| Canal | Distro |
|---|---|
| Pacote de sistema/repositório próprio do ecossistema | Fedora (base atual do Lyra OS) |
| AUR (`lyra-chord`) | Arch |
| RPM (openSUSE, se retomado no futuro) | openSUSE |

- Dois sub-pacotes gerados a partir do mesmo repositório: `lyra-chord-gtk` (dependência do meta-pacote GNOME do Lyra OS) e `lyra-chord-qt` (dependência do meta-pacote KDE, quando essa variante existir)
- Desktop entries (`org.lyraos.Chord.desktop` para cada frontend) incluem `Terminal=false` e categoria `System;TerminalEmulator;`

---

## 5. Integração com o Sistema

- **Terminal padrão do Lyra OS (edição GNOME):** já definido como GNOME Terminal por enquanto (decisão registrada separadamente) — o Chord é candidato a substituir isso quando estiver maduro o suficiente; não assumir substituição automática nesta especificação
- **Vega:** módulos que mostram output técnico podem oferecer "Abrir no Chord" como ação futura — não implementado nesta versão

---

## 6. Validação

- [x] `chord-core` compila como crate independente, sem nenhuma dependência de GTK ou Qt
- [ ] `chord-gtk` e `chord-qt` compartilham exatamente a mesma lógica de tema — alterar `palette.json` reflete nos dois sem tocar código de frontend (chord-qt ainda não implementado)
- [x] Abas, splits e atalhos funcionam identicamente em ambos os frontends (chord-gtk; chord-qt pendente)
- [x] Tema "Chord Dark" aplica a paleta ANSI corretamente em ambos (validar com teste de cores ANSI, ex. `curl parrot.live`) (chord-gtk; chord-qt pendente)
- [x] Configuração em `~/.config/chord/config.toml` é lida/escrita corretamente por ambos os frontends, sem depender de GSettings ou KConfig (chord-core; consumida por chord-gtk)
- [x] Carregar um tema externo (base16) funciona identicamente nos dois frontends (implementado em chord-core; chord-qt pendente)
- [ ] Build funciona nos canais de empacotamento definidos (§4)
- [ ] Nome "Chord" não gera confusão relevante com o protocolo Chord (DHT) em nenhum material de divulgação — checar antes do primeiro anúncio público

---

## 7. Fora de Escopo

- Multiplexação de sessão persistente (tmux-like)
- Integração profunda com Vega
- Sincronização de configuração entre máquinas
- Frontend TUI (não é um caso de uso claro para um emulador de terminal em si)

---

**Fim da especificação.**
