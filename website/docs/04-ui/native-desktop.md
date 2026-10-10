# GUI nativa — Petunia Studio

A janela é implementada em Qt Quick/QML, conectada a Rust pelo CXX-Qt 0.10.
O adaptador `petunia-desktop` depende da sessão headless e dos contratos
existentes. Core, Engine e Render continuam sem dependências Qt.

## Executar

Pré-requisitos: toolchain de `rust-toolchain.toml`, compiladores C/C++, SDK
Qt 6.8 com Core, Gui, Qml, Quick, Quick Controls 2, ferramentas QML,
módulos Quick Layouts/Dialogs/Window e plugin de imagem SVG. Em Linux,
use LLD, Gold ou Mold para preservar os registros do módulo QML.

```bash
./scripts/run-studio.sh
# equivalente: cargo run -p petunia-desktop --features native --bin petunia-studio
```

`QMAKE` pode apontar para o qmake6 de um SDK específico. As preferências ficam
no diretório de configuração do usuário (`XDG_CONFIG_HOME`/`.config`,
`APPDATA` ou `Library/Application Support`) sob `petunia-studio`.
`PETUNIA_CONFIG_DIR` permite isolar uma execução de teste.

## Fluxos implementados

- Janela com seletor de personas, barra contextual, ferramentas, canvas,
  painéis de camadas, transformação, cor e histórico, e barra de estado.
- Personas podem ser duplicadas, renomeadas, reordenadas, ocultadas e
  configuradas; preferências são persistidas separadamente do documento.
- Aparência clara, escura e alto contraste; densidade confortável/compacta;
  Phosphor/Tabler com contorno/preenchimento; escala de interface de 100–200%.
- Retângulo, elipse, seleção, movimento, caneta e edição vetorial usam o
  commit lane existente. Previews não incrementam revisão; Escape cancela
  o rascunho e Enter conclui uma linha aberta da caneta.
- Novo, abrir, salvar/salvar como PTND, exportar PNG, desfazer/refazer,
  duplicar, excluir, visibilidade e bloqueio têm tratamento de erro visível.
- Paleta de comandos informa por que uma ação está indisponível.
  Ferramentas de pintura e texto ainda sem controlador de janela aparecem
  indisponíveis; escolher uma persona não cria uma capacidade de edição.

## Fronteiras e dados

`StudioBackend` expõe somente snapshots JSON de apresentação, comandos
validados e imagens do renderer. Os caminhos dos diálogos são normalizados
no adaptador; arquivos remotos e URLs malformadas são recusados.
Operações destrutivas exigem ausência de gesto e descarte explícito de um
documento alterado. Sobrescrita só ocorre depois de confirmação no diálogo.
Preferências são validadas e gravadas atomicamente antes de publicar o estado.

Os SVGs são embarcados, com revisões e checksums em
`crates/petunia-desktop/assets/icons/manifest.json`. Variantes preenchidas
ausentes usam o contorno da mesma família, com registro no catálogo.

## Verificação

```bash
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy -p petunia-desktop --features native --all-targets -- -D warnings
qmltestrunner -input crates/petunia-desktop/qml/tests -platform offscreen
bash scripts/check-native-gui.sh
```

O último comando executa a janela real com backend Rust, salva e reabre um
documento, exporta PNG e captura temas/escala em DPI 1 e 2. Os artefatos ficam
em `work/gui-validation-*`. Esse teste usa o backend software do Qt e compara
81 pixels do canvas capturado com o frame Rust antes de aceitar cada imagem.
A captura diagnóstica só fica disponível com `--smoke-test`.

O estado e as evidências de execução são registrados em U01 no tracker.
Validação offscreen e atributos Accessible não certificam integração com
leitores de tela de cada sistema. QA com AT-SPI/NVDA/VoiceOver, hardware GPU,
dispositivos de caneta e empacotamento multiplataforma exigem execução nos
ambientes correspondentes.

## Evidência executada — 2026-10-10

Working tree sobre `6e2be4e1ce400e1b4642acc2de1007d7cd5e67c9`, Linux x86_64,
Rust 1.99.0, Qt 6.8.2, CXX-Qt 0.10.0.

| Gate | Resultado observado |
|---|---|
| Workspace Rust, sem feature native | PASS: 537 testes; zero falhas |
| Clippy workspace e adaptador native, todos os targets, warnings negados | PASS |
| QtQuickTest | PASS: 26 resultados, zero falhas |
| Lint QML | PASS: 11 componentes produtivos, sem avisos |
| Proveniência de ícones | PASS: 172 entradas; avisos MIT de ambas as famílias |
| Janela real com backend Rust | PASS: DPI 1 e 2; escuro/Phosphor outline, claro/Tabler fill e alto contraste/escala 200%; canvas confirmado por leitura de pixels |
| Arquivos e interação nativa | PASS: salvar/reabrir PTND, exportar PNG, undo/redo e proteção/cancelamento de caneta pendente |
| Leitores de tela, GPU e plataformas finais | NOT RUN: exigem os ambientes correspondentes |

A execução final gerou `work/gui-validation-N53RiU`. Esses artefatos são
locais e regeneráveis pelo script; não substituem QA das plataformas finais.
O reduzido movimento manual é persistido; a descoberta da preferência do
sistema e a posição de painéis flutuantes entre sessões ainda estão pendentes.
U01 permanece **IN PROGRESS** para os gates de produto restantes.
