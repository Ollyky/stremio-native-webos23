# StremioX como referência para Stremio Native (webOS 23)

Auditoria estática feita em 2026-10-29. O repositório local foi consultado somente para leitura.

## Identidade e versões conferidas

| Projeto/ref | Evidência |
|---|---|
| [yvdjee/stremiox](https://github.com/yvdjee/stremiox) | É um fork de [VortXTV/VortX](https://github.com/VortXTV/VortX), não um app webOS. A branch `main` estava em `341c87520f8c011b6a995557a1008bb6e123314b`; a outra branch pública observada era `0.3.0-beta5`. A tag mais recente é `v0.3.0-beta13`. A API não listou GitHub Releases publicados nesse repositório na consulta desta auditoria. |
| [Morpheus235/stremio-native](https://github.com/Morpheus235/stremio-native) | `main` em `e89a1c19bd0443ef1a5570892a7a462605d3ed14`, sem PRs abertos visíveis na consulta. |
| [Ollyky/stremio-native-webos23](https://github.com/Ollyky/stremio-native-webos23) | `main` no mesmo commit `e89a1c19bd0443ef1a5570892a7a462605d3ed14). O PR [#1](https://github.com/Ollyky/stremio-native-webos23/pull/1) continua aberto entre `codex/fix-player-froststream-subtitles-home` e `build-ipk/test-paleta`; não foi incorporado a esta proposta. A branch anterior `codex/native-ux-improvements` também foi preservada. |

A cópia local em `/mnt/SAMSUNG-SSD/STREMIO/stremio-native` estava limpa na branch `codex/native-ux-improvements`, com HEAD `e89a1c19bd0443ef1a5570892a7a462605d3ed14), igual ao `main` remoto. Nenhum arquivo local foi alterado.

## O que o StremioX realmente implementa

- O README descreve apps nativos para Apple TV, iPhone, iPad e Mac, usando `stremio-core`, SwiftUI e libmpv via MPVKit. Para Apple TV há builds Full e Lite; o Full traz NodeMobile e `server.js` para torrents. O projeto não é uma base de webOS.
- A Home de tvOS põe Continue Watching antes dos catálogos e atualiza o backdrop/hero conforme o item focado. `FocusedItemModel` semeia e pré-carrega metadados; `SharedUI.swift` usa cache de artwork em memória e em disco. Esse caminho faz trabalho extra e não deve ser transplantado para uma TV webOS sem medir memória e solicitações.
- A tela de detalhes de tvOS apresenta temporadas/episódios e fontes agrupadas por addon. `StreamRanking.swift` classifica streams usando texto do addon, tipo/cache, qualidade e preferências. Isso descreve ranking baseado nos dados disponíveis, não medições do player.
- A parte Android usa Compose, mas o próprio `DetailScreen.kt` declara que o botão Watch e as linhas de fonte ainda são inertes enquanto engine/player não chegam. O README também marca Android e desktop como desenvolvimento ativo e não distribuídos. Não há evidência de execução em webOS 23.
- Os workflows públicos constam como Android e Apple/tvOS. O README exige macOS/Xcode, XcodeGen e Rust nightly para os apps Apple; MPVKit chega por Swift Package Manager, e o script de dependências do servidor obtém NodeMobile e o `server.js` que não é versionado no repositório. Esse processo não produz IPK webOS.

Referências de código e documentos:

- [README do StremioX](https://github.com/yvdjee/stremiox/blob/main/README.md)
- [Home tvOS](https://github.com/yvdjee/stremiox/blob/main/app/SourcesTV/HomeView.swift)
- [Detalhes e fontes tvOS](https://github.com/yvdjee/stremiox/blob/main/app/SourcesTV/DetailView.swift)
- [Cache, foco e backdrop compartilhados](https://github.com/yvdjee/stremiox/blob/main/app/SourcesTV/SharedUI.swift)
- [Ranking de fontes](https://github.com/yvdjee/stremiox/blob/main/app/SourcesShared/StreamRanking.swift)
- [Tema e foco](https://github.com/yvdjee/stremiox/blob/main/app/SourcesShared/Theme.swift)
- [Tela Android de detalhes](https://github.com/yvdjee/stremiox/blob/main/android/app/src/main/kotlin/com/stremiox/android/ui/screens/DetailScreen.kt)
- [Design system](https://github.com/yvdjee/stremiox/blob/main/DESIGN.md), [roadmap](https://github.com/yvdjee/stremiox/blob/main/ROADMAP.md), [CI Android](https://github.com/yvdjee/stremiox/blob/main/.github/workflows/android.yml) e [workflow tvOS](https://github.com/yvdjee/stremiox/blob/main/.github/workflows/release-tvos.yml)
- [Commit atual da branch main](https://github.com/yvdjee/stremiox/commit/341c87520f8c011b6a995557a1008bb6e123314b), [tag beta13](https://github.com/yvdjee/stremiox/tree/v0.3.0-beta13)
- [PRs públicos abertos observados](https://github.com/yvdjee/stremiox/pulls)

## Comparação com o Stremio Native oficial

| Área | StremioX | Stremio Native em `e89a1c19` | Avaliação para webOS 23 |
|---|---|---|---|
| Home | Hero/backdrop acompanha o foco; pré-carrega dados e artwork. | Continue Watching e catálogos são linhas nativas, com carregamento por viewport e cancelamento fora da tela. | O padrão de hero é uma ideia visual, mas seu enriquecimento/cache acrescenta memória e tráfego. Não há evidência que justifique copiá-lo. Preservar a Home atual. |
| Detalhes e temporadas | Página cinematográfica, seletor e episódios. | Página nativa com episódios por temporada, estado de carregamento/erro e links de metadados. | Não foi demonstrada uma lacuna funcional transferível. |
| Fontes | Lista completa agrupada por addon, ranking e seletor de qualidade. | Também agrupa por addon, mantém ordem original como padrão e oferece ordenação/preferências. Linhas usam nome próprio, nome do release e fatos/chips. | Os conceitos centrais já existem. Não transplantar ranking, SwiftUI ou apresentação sem uma regressão reproduzível. |
| Foco | Apple TV usa foco e elevação/glow próprios do tvOS. | `TableView` usa pílula roxa animada por mola; a linha focada expande para mostrar texto e chips completos sem trocar ID/ordem. Há testes de transição, expansão, truncamento e coluna lateral. | O modelo nativo já é apropriado ao controle remoto; códigos de controle Apple não substituem os códigos webOS documentados em [remote-keys.md](https://github.com/Morpheus235/stremio-native/blob/main/docs/remote-keys.md). |
| Contraste | Tema próprio de tvOS e contraste de foco. | `FOCUS_FILL` usa roxo Stremio. Texto principal branco passa 5:1; texto secundário sobre o foco usa branco a 78% e o teste aceita 3:1. | Ajuste pequeno proposto abaixo: elevar só o texto secundário no estado focado a pelo menos 4,5:1. A melhoria matemática é confirmável; aparência final exige TV/captura real. |
| Parser e compatibilidade | Ranking extrai sinais de texto do addon e os memoiza. | `stream_facts` já centraliza parsing, deixa campos ausentes vazios e possui testes contra várias entradas. | Não portar parser/ranking Apple. Não atribuir ao player valores que vieram do addon. |
| Erros e fluxo | Há issues/PRs públicos recentes para navegação de episódio e crashes de áudio/tela cheia, em plataformas Apple. | O roadmap nativo registra seu próprio problema de freeze ao sair do player, além do passe de contraste da lista de fontes/episódios. | Os relatos Apple não confirmam causas em webOS. Playback fica fora desta mudança. |

Arquivos nativos conferidos: [instruções AGENTS.md](https://github.com/Morpheus235/stremio-native/blob/main/AGENTS.md), [roadmap](https://github.com/Morpheus235/stremio-native/blob/main/docs/roadmap.md), [detail.rs](https://github.com/Morpheus235/stremio-native/blob/main/crates/screens/src/detail.rs), [table.rs](https://github.com/Morpheus235/stremio-native/blob/main/crates/ui/src/table.rs), [theme.rs](https://github.com/Morpheus235/stremio-native/blob/main/crates/ui/src/theme.rs), [stream_facts.rs](https://github.com/Morpheus235/stremio-native/blob/main/crates/stremio/src/stream_facts.rs), [home.rs](https://github.com/Morpheus235/stremio-native/blob/main/crates/screens/src/home.rs) e [CI](https://github.com/Morpheus235/stremio-native/blob/main/.github/workflows/ci.yml).

## Melhorias priorizadas

1. **Maior contraste nas informações secundárias da linha focada** — impacto potencial alto à distância; risco funcional baixo; compatibilidade webOS 23 alta, pois usa só o token já existente. É a única mudança implementada nesta primeira fatia. Afeta texto secundário de linhas focadas em tabelas compartilhadas, não apenas fontes; o teste matemático cobre o token.
2. **Manter expansão/truncamento, foco por identidade e carregamento incremental** — a implementação nativa e testes já cobrem esses pontos. Sem alteração necessária.
3. **Não portar o hero vivo/cache do StremioX agora** — ganho visual não verificado no app da TV; custo potencial de rede/memória e risco de recriação de linhas/foco. Só reconsiderar com captura real e medição no hardware.
4. **Não portar ranking/fluxo Android** — a UI Android ainda é incompleta; não traz evidência de controle remoto webOS nem de reprodução integrada.
5. **Stats for nerds e novos filtros** — sem evidência suficiente nesta referência para propor mudanças além do que o app já tem. Devem ser guiados pelas métricas realmente expostas pelo player e por testes do código nativo.

## Implementação e validação

A mudança limitada troca o alfa do token secundário sobre a pílula focada de 0,78 para 1,0 e atualiza o teste do limite de contraste de 3:1 para 4,5:1. Não altera cores fora desse estado, layout, fontes, foco, controle remoto, rede, carregamento, addons ou pipeline de playback.

A branch de trabalho foi criada em `Ollyky/stremio-native-webos23` a partir do `main` do fork, que estava idêntico ao `main` oficial em `e89a1c19bd0443ef1a5570892a7a462605d3ed14`. Nenhuma alteração do PR #1 foi incorporada.

Os workflows `beta-ipk.yml` e `publish-tested-beta.yml` são preparados para construir um IPK Beta com commit/versão/SHA-256 e, depois, anexar o mesmo artefato a uma pré-release somente após confirmação textual explícita do teste na TV. Nenhuma release é criada por esta proposta. A execução, o checksum e a disponibilidade de um IPK só serão informados se o Actions confirmar o artefato. A validação visual final ainda depende de uma captura ou teste na TV; isso não substitui a cobertura automatizada.
