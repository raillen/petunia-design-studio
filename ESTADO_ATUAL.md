# Estado atual — Petunia Design Studio

**Data:** 2026-10-03
**Branch:** `work/v1-native-cmyk-2026-10-03`
**Contrato atual:** [ADR-011](docs/developers/adr/ADR-011-native-cmyk-raster.md)
**Roadmap:** [MVP/v1](docs/pt/developers/implementation-roadmap-2026-09-30.md)

A continuação implementa CMYKA 8/16-bit, perfil ICC por camada, atribuição/conversão imutável, schema 6/índice 3, pintura/preenchimento nativos, importação TIFF editável, prévia ICC descartável, inspeção TAC/separações de camada, TIFF de camada e PDF CMYK ICC. Interface e CLI usam os limites existentes de mutação/undo/publicação. A prova RGB é bloqueada para tinta nativa; não se confunde exibição com prova de impressão.

**Validação final:** `cargo xtask gauntlet` passou com **930 testes**, zero falhas/testes ignorados, formatação, Clippy estrito, arquitetura, conformidade CLI e quatro projetos de exemplo. `cargo xtask ui-gauntlet` passou **64 testes desktop**, incluídos no total. Documentação: **33/33 páginas EN/pt-BR**, sem links quebrados. Quatro PDFs de camada/imagem 8/16-bit passaram na conferência independente pypdf/Poppler das tintas, alpha e ICC originais. Comandos, logs, capturas e hashes estão em [native-cmyk-v1.json](docs/public/implementation/native-cmyk-v1.json). Os 904 testes em [mvp-v1-corrections.json](docs/public/implementation/mvp-v1-corrections.json) pertencem à baseline ADR-010 integrada no PR #5. Os testes foram executados após a implementação, com correções decorrentes da validação final.

O PDF regular exige um perfil de impressão comum entre as superfícies/camadas/imagens CMYK incluídas; perfis diferentes exigem conversão explícita. TIFF e inspeção TAC/separações têm escopo de camada autoral.

**MVP e V1 não estão concluídos como release.** Permanecem prova/composição/separações de página com overprint, paridade spot, política DeviceLink/preto, PDF/X-4 e tipografia avançada. Aceitação de usuários, instalação, tablet/IME físico, acessibilidade, calibração e sessões longas exigem ambiente representativo.
