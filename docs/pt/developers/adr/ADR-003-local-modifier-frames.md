# ADR-003: Frames locais persistentes dos modificadores

- **Status:** Aceito
- **Data:** 2026-10-01
- **Escopo:** Milestone Required (MVP)
- **Afeta:** document, application, shell e pacotes nativos; schema 3
- **Atualiza:** decisão 4 e versão do schema do ADR-002; os contratos de caminhos locais, integridade e publicação continuam aplicáveis

## Contexto

Os caminhos do schema 2 eram locais, mas parâmetros de modificadores usavam coordenadas do pai. Subtrair a origem atual de bounds a cada leitura deixava crop, perspectiva e transparência para trás ao mover. Redimensionar alterava sua relação com a fonte. Fazer bake da cadeia inteira e remover somente contours podia reaplicar a geometria restante. Mudar a origem de bounds durante bake deslocava geometria rotacionada e máscaras.

## Decisão

1. Persistir `ModifierSpace::Local { reference_size: [width, height] }` por entrada. `Parent` é descritor explícito de entrada/compatibilidade. Normalizar uma vez pelos bounds atuais, inclusive entradas desativadas. Posicionamento/escala não reescrevem parâmetros ou tamanho de referência; IDs/ordem sobrevivem à normalização.
2. Ler schemas 1/2 migrando marcadores ausentes/Parent pela convenção documentada. Schema 3 exige marcadores locais explícitos. Rejeitar versões desconhecidas/frames inválidos. Saves escrevem schema 3, para leitores antigos rejeitarem em vez de interpretar incorretamente. Versões do pacote/payload precisam coincidir antes da migração.
3. Para geometria, transformar o contorno local atual à referência de cada entrada, avaliar e retornar. Escala desigual redimensiona contour em ambos os eixos sem inventar distância média. Tolerância de flattening de crop/warp considera a escala. Caminhos e receitas originais continuam editáveis.
4. Transparência transforma a amostra à referência antes da projeção. Projetar nos endpoints redimensionados é incorreto sob escala desigual. Sampling preserva ordem e comportamento de stops duplicados sem alocar/ordenar por amostra. APIs locais/mundiais estão disponíveis; composição de ancestrais é separada.
5. Crop vazio produz contorno avaliado vazio e mantém a fonte. Warp/offset inválido mantém o fallback existente; limites/diagnósticos geométricos mais amplos continuam necessários.
6. `BakeGeometry` explícito consome geometria habilitada e faz rebase de transparência/entradas desativadas, preservando escala por eixo. A nova posição considera a rotação. Preparar e validar candidato/ChangeSet antes de publicar; resultados vazios continuam vazios. `BakeContour` consome somente prefixo de contour e preserva modificadores seguintes. Se outra geometria preceder contour, falhar com motivo para usar `BakeGeometry`; não consumir outro efeito silenciosamente.
7. Converter entradas mundiais das ferramentas pelas transformações do objeto/ancestrais. Overlay/preview/commit de perspectiva usam frames locais; cantos rotacionados não arrastados permanecem iguais. Crop alinhado ao mundo que não seja retângulo local falha com requisito de máscara poligonal. Essas máscaras continuam pendentes; não substituir por AABB envolvente.
8. Validar cadeias substitutas sem clonar o objeto/imagem inteiro. Criar guia aloca ID local único em comando de domínio, substituindo IDs em milissegundos que colidiam em gestos consecutivos da régua. Histórico reaplica identidades armazenadas.

## Evidência e limites

Testes focais cobrem migração, movimento/escala desigual, escala de contour, opacity diagonal, ancestrais rotacionados, entradas desativadas, frames inválidos, crop vazio, bake rotacionado/parcial, branches e round trips de pacote. Fixture de ponteiro cobre perspectiva rotacionada; fixtures de régua cobrem guias consecutivas. `cargo xtask migrations` inclui frames/pacotes. Evidência gerada: [contracts.json](/implementation/contracts.json), [checks.json](/implementation/checks.json).

Isso conclui a parte de frames de modificadores de M0, não M0 ou o MVP. Cena comum, frames de paints, opacity espacial em preview/export, recursos COW, quotas, workers, camadas de pixels/máscaras, recovery e aceite de Linux/acessibilidade continuam abertos. Opacity pelo centro no export continua aproximação. Consulte o [registro de execução](/pt/developers/implementation-progress).
