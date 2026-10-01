# ADR-002: Caminhos locais e integridade na publicação

- **Status:** Aceito
- **Data:** 2026-10-01
- **Escopo:** Milestone Required (MVP)
- **Afeta:** document, foundation, application, shell, raster e IO; schema nativo 2

A decisão de frame dos modificadores e a versão do schema são atualizadas pelo [ADR-003](/pt/developers/adr/ADR-003-local-modifier-frames); os demais contratos continuam aplicáveis.

## Contexto

Os pontos de `Path` no schema 1 usavam coordenadas do pai, mas o resolvedor mundial os rejeitava como ambíguos. Mover apenas bounds podia deixar os pontos para trás. Comandos independentes de forma/enquadramento podiam normalizar a geometria no frame errado. Publicar uma transação podia sobrescrever comandos intervenientes; falhar no replay podia perder entradas e alterar parte do documento.

## Decisão

1. Persistir `LocalPath { path, reference_size }`. Os pontos originais são locais e não mudam durante alterações de posicionamento. A avaliação local escala pelo tamanho atual de bounds dividido pelo tamanho de referência; o posicionamento mundial aplica as matrizes do objeto e ancestrais. Receitas paramétricas continuam editáveis.
2. Manter `Path` como descritor de entrada/compatibilidade explicitamente no frame do pai. Mutators e leitura do schema 1 traduzem pela origem negativa de bounds e armazenam `LocalPath`. Não inferir o frame pela posição dos pontos. Rejeitar schemas desconhecidos. Versões do manifesto/documento precisam coincidir antes da migração; saves escrevem schema 2.
3. `SetPath` substitui caminho e enquadramento atomicamente. Comandos de criação definem bounds antes da forma. Uma entrada apenas de caminho, sem bounds, deriva um frame positivo da geometria; um caminho vazio exige frame explícito. `to_path` é projeção de compatibilidade no frame do pai, nunca a fonte canônica persistida.
4. Parâmetros dos modificadores legados mantêm o frame do pai documentado. Avaliação local traduz endpoints/quads/origem de crop para o frame local, preservando a cadeia editável. Uma migração explícita dos frames dos modificadores ainda precisa fazer todos os parâmetros acompanharem posição e escala.
5. Validar na leitura/publicação: frames finitos e positivos, métricas das formas, opacity, strokes, stops de gradientes, IDs/parâmetros de modifiers, IDs de superfícies, IDs globais de objetos, propriedade recíproca, referências na mesma superfície, pais acíclicos, campos de fontes de dados e bindings. A validação de grafo é iterativa. Excluir um caminho referenciado desanexa texto e bindings com mudanças reversíveis. Escritas de aparência validam todas as variantes atuais de efeitos/ajustes, IDs locais e limites numéricos documentados antes de tocar campos legados ou stack, inclusive entradas desativadas. Curvas admitem Y invertido, mas exigem X crescente. Acessos mutáveis ao documento/objeto/superfície e escrita do schema ficam restritos ao crate document; métodos de bypass sem uso foram removidos. Decoding serde bruto continua entrada não confiável de baixo nível e exige validação nas fronteiras de persistência/publicação; essa validação rejeita schemas e caminhos no frame do pai ainda não migrados.
6. Transações retêm a baseline e rejeitam documentos alterados durante o preview. Falhas de staging e replay não publicam mudanças. Entradas de undo/redo só mudam de pilha após replay e validação bem-sucedidos. A identidade do ponto salvo acompanha o histórico do conteúdo, independentemente das revisões monotônicas do cache.
7. Histórico padrão retém até 1000 entradas e estimativa de 512 MiB de payload codificado entre undo/redo. Serialização contadora evita alocar outro buffer. Edições maiores falham antes da publicação; a retenção não remove silenciosamente o undo da edição atual. A estimativa **não** limita RSS.
8. Saves usam temporários exclusivos no diretório de destino, JSON transmitido ao ZIP, sincronização do arquivo, substituição atômica e sincronização do diretório em Unix. A propriedade do temporário garante limpeza. Leitura limita manifesto a 64 KiB, JSON do documento a 256 MiB e arquivo a 10.000 entradas; entradas centrais precisam ser únicas. Recursos binários/vinculados, decoding incremental e recovery são trabalho subsequente.
9. APIs raster normalizadas aceitam/retornam RGBA straight; armazenamento respeita o alpha mode declarado. Canais de 16 bits usam little-endian. Skia recebe bytes straight como Unpremul. Dabs inválidos ou acima do orçamento síncrono falham antes de tocar tiles; endereços fora do intervalo não podem sofrer wrap.

## Consequências e evidência

Migração, preservação da fonte ao mover/escalar, substituição/revert atômicos, grafos inválidos, cache antigo, buracos em paths compostos, conflitos, falhas de undo/redo, branches/ponto salvo, orçamento, saves concorrentes, limites de pacote, persistência de formatadores, rejeição de aparência/layout, histórico de inclusão no export e equivalência de alpha possuem testes de integração focais. Doctests compile-fail provam que o acesso mutável externo a objetos/superfícies está indisponível. `cargo xtask migrations` executa as fixtures de migração/pacote. A [referência gerada](/implementation/contracts.json) lista os arquivos; o [registro de execução](/pt/developers/implementation-progress) registra resultados e milestones pendentes.

Buffers imutáveis dos caminhos canônicos são compartilhados por snapshots com Arc; substituições preservam a identidade independente da fonte. Primitivas atômicas auditadas fazem staging direto; comandos compostos mantêm cópia protetora. A implementação ainda clona contêineres e recursos de imagens do documento. Isso aumenta custo e memória de mutações em projetos grandes. Recursos COW, precondições/revisões de operações e workers limitados precisam substituir as cópias com equivalência medida; este ADR não declara M0/MVP ou V1 completos. Validação humana de Linux/pen/IME/acessibilidade e impressão continua aberta.

O anexo da auditoria conserva a evidência da baseline. Cadernos históricos com outra GUI não sobrepõem os manifests atuais Freya/Skia; a arquitetura do site agora informa o runtime implementado. A reconciliação ampla do authority map continua no backlog explícito.
