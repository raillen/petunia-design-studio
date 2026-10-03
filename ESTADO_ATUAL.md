# Estado atual — Petunia Design Studio

**Data:** 2026-10-03
**Branch:** `work/mvp-v1-corrections-2026-10-03`
**Contrato atual:** [ADR-010](docs/developers/adr/ADR-010-text-histogram-icc-and-pdf.md)
**Roadmap:** [MVP/v1](docs/pt/developers/implementation-roadmap-2026-09-30.md)

A implementação recebeu correções de histórico/coalescência, texto no canvas por grafemas/IME, histogramas da composição em workers, alpha/fundos, persistência ICC schema 5, conversão LittleCMS e PDF por cena com fontes/imagens/clips. Recursos indisponíveis continuam desabilitados com razões. Testes são executados somente na fase final, conforme o pedido do usuário; a evidência concreta é [mvp-v1-corrections.json](docs/public/implementation/mvp-v1-corrections.json).

**Validação final:** 904 testes do workspace passaram, sem falhas ou testes ignorados. Formatação, Clippy estrito, arquitetura, conformidade CLI, quatro fixtures do MVP e documentação bilíngue passaram. O gauntlet de UI passou 62 testes, já incluídos no total. Logs, hashes e capturas acompanham o registro de execução.

Este estado substitui as afirmações antigas de que todo recurso conectado estava comprovado. **Não declara o MVP ou a v1 concluídos como release.** Aceitação das quatro tarefas reais, instalação, tablet/IME físico, acessibilidade, desempenho de sessões longas e calibração exigem ambiente e usuários representativos. A v1 ainda requer raster CMYK com quatro tintas nativas, separações/TAC/DeviceLink, overprint/spot completos e PDF/X-4 com validação independente de impressão.
