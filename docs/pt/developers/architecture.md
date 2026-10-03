# Arquitetura

Visão estilo C4 do Petunia Design Studio: contextos, contêineres, componentes e o caminho de mutação no nível do código.

Contrato atual de persistência/publicação: [ADR-002](/pt/developers/adr/ADR-002-local-path-and-integrity), com [referência gerada de fontes/testes](/implementation/contracts.json). O [registro de execução](/pt/developers/implementation-progress) distingue correções implementadas de milestones pendentes.

Renderização/apresentação compartilhadas: [ADR-006](/pt/developers/adr/ADR-006-shaped-text-and-canvas-preview) conecta contornos de glifos e workers com fontes imutáveis ao canvas. Saída de pixels não validada; edição/estilos de texto, tiles e aceitação completa seguem abertos.

## Nível 1 — Contexto do sistema

```mermaid
flowchart TB
  USER[Designer / Fotógrafo]
  AGENT[Agente de IA / script]
  PETUNIA[Petunia Design Studio]
  FILES[(.ptnd · SVG · PDF · PNG<br/>fontes CSV/TSV/JSON)]
  USER <--> PETUNIA
  AGENT <-->|MCP · CLI · Lua| PETUNIA
  PETUNIA <--> FILES
```

Humanos dirigem a shell Freya/Skia; agentes e scripts dirigem a mesma superfície de capacidades via MCP, CLI e Lua — sem API privilegiada paralela.

## Nível 2 — Contêineres

```mermaid
flowchart TB
  APP[App desktop Freya/Skia<br/>petunia-design]
  CLI[CLI headless<br/>petunia-design-cli]
  CORE[Domínio + serviços<br/>16 crates]
  STORE[(Pacote .ptnd<br/>ZIP atômico)]
  APP <--> CORE
  CLI <--> CORE
  CORE <--> STORE
  APP -->|SVG · PDF · PNG| OUT([Artefatos de exportação])
  CLI -->|SVG · PDF · PNG| OUT
```

## Nível 3 — Componentes (a fronteira da ponte)

```mermaid
flowchart LR
  subgraph GUI ["Lado GUI (Freya/Skia)"]
    VW[ViewportCamera<br/>0,1% – 25600%]
    PAN[Painéis<br/>Camadas · Propriedades<br/>Histórico · DataMerge]
    TM[ToolManager<br/>35 ToolKinds]
  end
  subgraph BRIDGE ["AubrietaGuiBridge (fachada)"]
    AQ[ActionQueryPort]
    CP[CommandPort]
    PP[PropertyPort]
    DQ[DocumentQueryPort]
    SP[SelectionPort]
    IP[InspectionPort]
  end
  subgraph CORE2 ["Núcleo da aplicação"]
    SES[DocumentSession<br/>documento + histórico privados]
    HIST[History<br/>undo / redo]
    MUT[DocumentMutator]
  end
  TM --> BRIDGE
  PAN --> BRIDGE
  VW --> BRIDGE
  BRIDGE --> SES
  SES --> HIST
  SES --> MUT
```

Regras:

- **Via única de mutação** — `DocumentSession` detém `document`/`history` privados; a GUI nunca segura um handle mutável.
- **DTOs cruzam a ponte** — view-models e `TransformPreview` são neutros a toolkit; pixels de overlay nunca vazam tipos de domínio.
- **Registros de capacidade** — ferramentas, painéis, efeitos, importadores e fontes de dados compõem-se via registros; capacidade ausente é estado normal desabilitado com motivo, nunca pânico.

## Nível 4 — Caminho de um gesto no código

```ts
// Idêntico em todo binding de linguagem; comandos de terminal seguem iguais,
// só comentários são traduzidos (veja o guia de Traduções).
PointerDown  // hit-test, captura revisão, só preview
PointerMove  // atualiza DTO TransformPreview (sem escrita no histórico)
PointerUp    // transact: Ação -> Comando -> DocumentMutator -> ChangeSet
Undo         // History reverte o ChangeSet único (um gesto = um undo)
```

## Invariantes-chave

1. Crates de domínio nunca importam tipos de toolkit GUI (imposto em CI).
2. Só IDs estáveis tipados (`ObjectId`, `SurfaceId`, `ResourceId`…) — nunca índices de Vec ou ponteiros.
3. Base vs. avaliado: ferramentas editam paths base; render/hit/seleção leem `evaluated_path()` (EffectChain aplicada).
4. Sem UI falsa: comportamento não declarado é implementado, desabilitado-com-motivo, oculto ou marcado experimental.

Decisões por trás deste desenho: [Índice de ADRs](/pt/developers/adr/) · [ADR-001](/pt/developers/adr/ADR-001-effect-chain).

## Atualização do contrato em 2026-10-03

[ADR-010](/pt/developers/adr/ADR-010-text-histogram-icc-and-pdf) governa rascunhos de texto, histogramas da composição, recursos ICC imutáveis (schema 5), atribuição de perfis com precondições e o subconjunto fiel de PDF. A aceitação de impressão profissional e hardware permanece aberta.
