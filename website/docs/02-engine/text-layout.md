# Text + Layout Engine

Texto possui três problemas diferentes: **shaping**, **line layout** e **page/layout flow**.

- shaping decide quais glifos devem representar os caracteres;
- line layout decide onde quebrar e posicionar linhas;
- page/layout flow distribui o texto entre frames, colunas e páginas.

Separar essas etapas evita colocar toda tipografia em uma única função difícil de testar.

## Shaping

**Text shaping** transforma texto Unicode em uma sequência de glifos posicionados.

Um **caractere** é informação textual. Um **glifo** é a forma visual escolhida pela fonte.

A relação não é sempre 1 caractere = 1 glifo.

Exemplo:

~~~text
"f" + "i"
   ↓ ligature
"ﬁ"
~~~

Uma **ligature** substitui uma sequência por um glifo combinado quando a fonte e as regras tipográficas permitem.

**Kerning** ajusta a distância entre pares específicos de glifos, como “A” e “V”.

### Stack definida

A v0.1 usa uma pipeline Rust pequena e especializada:

- `rustybuzz` para shaping OpenType;
- `unicode-segmentation` para grapheme/word boundaries conforme UAX #29;
- `unicode-bidi` para o Unicode Bidirectional Algorithm conforme UAX #9;
- `unicode-linebreak` para oportunidades de quebra conforme UAX #14;
- `ttf-parser` para metadata, permissions, variable-font metadata e extração de outlines;
- `hypher` para hifenização baseada em padrões de idioma, atrás de adapter Petunia;
- `fontdue` apenas para rasterização de cobertura de glifos, não para shaping nem extração autoral de outlines.

Essas bibliotecas ficam atrás de tipos Petunia. IDs de glifo, structs de fonte e buffers externos não entram no Core.

### Font outline backend

`ttf-parser` é o parser definido para metadata e outlines de fontes na v0.1.

Ele é usado atrás de adapter para:

- family/style metadata;
- métricas;
- embedding permissions;
- variation axes;
- glyph outline extraction;
- bounding boxes;
- identificação de glyphs coloridos/raster/SVG quando necessário.

A API Petunia nunca expõe `ttf_parser::Face`, `GlyphId` ou `OutlineBuilder` como modelo persistente.

Outline extraído é Derived State até um Command explícito como `Convert Text to Curves`. Font permissions precisam ser respeitadas em embed/package/export.

### Hyphenation backend

A v0.1 usa `hypher` como backend inicial de hifenização.

Razões:

- implementação pequena e focada;
- padrões embutidos e determinísticos;
- suporte a múltiplos idiomas, incluindo português;
- sem necessidade de carregar dicionário arbitrário durante o layout;
- fica atrás de uma interface Petunia, portanto não contamina o modelo autoral.

O backend implementa o contrato canônico `HyphenationProvider` definido na seção de Hyphenation abaixo.

`LanguageTag` continua BCP 47 no Core. O adapter mapeia tags suportadas para o idioma do backend.

Idioma sem padrão disponível não produz hifenização automática; line breaking continua funcionando sem inventar pontos de quebra.

Atualizar os padrões/backend pode alterar oportunidades de hifenização. Isso exige corpus de regressão e, quando afetar documentos de layout fixo de forma incompatível, `OperationSemanticVersion` do layout.

### Contrato Unicode

A v0.1 segue os standards Unicode como fonte de semântica, em vez de inventar regras locais:

- **UAX #29 — Unicode Text Segmentation** para grapheme e word boundaries;
- **UAX #9 — Unicode Bidirectional Algorithm** para BiDi;
- **UAX #14 — Unicode Line Breaking Algorithm** para oportunidades de quebra;
- **BCP 47** para tags de idioma, usadas por shaping, fallback e hyphenation quando relevante.

**UAX** significa *Unicode Standard Annex*: documento normativo complementar ao Unicode Standard.

**BCP 47** define a sintaxe de tags como `pt-BR`, `en-US` e `ar`.

A versão Unicode suportada precisa ser tratada como dependência técnica versionada. Atualizar crates Unicode pode alterar boundaries ou classificação de caracteres; por isso upgrade exige corpus/regression tests de texto.

O documento persiste texto e tags sem “normalizar para acompanhar a biblioteca” durante load.

### Normalização Unicode

Petunia **não normaliza automaticamente o conteúdo textual** para NFC/NFD ao salvar.

Duas sequências Unicode visualmente equivalentes podem possuir significado técnico diferente para edição, interoperabilidade ou fonte.

Normalização só acontece quando:

- um algoritmo específico exige uma representação temporária;
- a conversão é Derived State;
- ou existe Command explícito do usuário.

Shaping recebe a source original conforme o contrato da biblioteca.

### Indexação

`TextRange` autoral usa offsets UTF-8 definidos no Core.

O Engine pode construir índices derivados para:

~~~text
byte offset
↔ grapheme index
↔ code point position
↔ cluster mapping
~~~

Esses mapas são caches/Derived State.

Nunca persistir glyph cluster indices como substitutos dos ranges Unicode autorais.

O workspace já usa `rustybuzz`, que realiza shaping considerando:

- fonte;
- script;
- direção;
- features OpenType;
- ligatures;
- kerning.

Output derivado:

~~~rust
pub struct GlyphRun {
    pub font: ResolvedFontId,
    pub glyphs: Vec<PositionedGlyph>,
    pub source_range: TextRange,
}
~~~

Glyph IDs e posições são derivados. O documento guarda texto e intenção tipográfica, não o resultado de shaping.

## Script

**Script** é o sistema de escrita usado por um trecho de texto, como Latin, Arabic, Devanagari ou Cyrillic.

Scripts diferentes possuem regras diferentes de conexão, direção e substituição de glifos.

Por isso o Engine precisa detectar ou receber o script correto antes do shaping.

## Unicode e grapheme clusters

Unicode armazena **code points**, mas o usuário percebe **caracteres visuais**.

Um **grapheme cluster** é a unidade que normalmente corresponde ao que uma pessoa entende como “um caractere” ao mover o cursor ou apagar texto.

Exemplo conceitual:

~~~text
letra base + acento combinante
          ↓
um grapheme visual
~~~

Cursor, seleção e backspace não devem separar combinações que formam um único grapheme quando as regras Unicode dizem que elas pertencem juntas.

## BiDi

**BiDi** significa *Bidirectional Text*: algoritmo que organiza corretamente texto contendo direções diferentes.

Exemplo:

~~~text
texto em português + trecho em árabe + número
~~~

A ordem armazenada na string e a ordem visual na tela podem ser diferentes.

O Petunia precisa aplicar o algoritmo Unicode Bidirectional antes de posicionar runs visuais.

Shaping e BiDi são relacionados, mas não são a mesma etapa.

## Line breaking

**Line breaking** decide em quais posições uma linha pode ser quebrada.

Não basta procurar espaços. Unicode possui regras para pontuação, ideogramas, combinações e outros scripts.

Pipeline conceitual:

~~~text
texto
 ↓
segmentação em graphemes
 ↓
BiDi / runs direcionais
 ↓
shaping
 ↓
oportunidades de quebra
 ↓
medição das linhas
 ↓
layout final
~~~

A ordem concreta pode ser ajustada conforme a biblioteca usada, mas cada responsabilidade deve permanecer explícita.

## Hyphenation

**Hyphenation** é a divisão de uma palavra entre linhas usando hífen.

Ela depende de idioma.

Exemplo:

~~~text
documentação
      ↓
documen-
tação
~~~

O Engine não deve aplicar regras de português a um parágrafo marcado como inglês.

A v0.1 trata hyphenation como serviço opcional por idioma atrás do contrato canônico `HyphenationProvider`, implementado inicialmente por `hypher`.

~~~rust
pub trait HyphenationProvider {
    fn opportunities(
        &self,
        language: LanguageTag,
        word: &str,
    ) -> HyphenationOpportunities;
}
~~~

O layout funciona corretamente sem dicionário, apenas com oportunidades normais de line break. Quando existe dicionário, ele adiciona pontos de hifenização. Isso impede que a ausência de um pacote linguístico torne o Text Engine inválido.

Dicionários são recursos versionados, não lógica embutida em QML.

## Font resolution

`FontRef` representa a intenção persistente do documento.

O Engine resolve essa referência para uma fonte realmente disponível:

~~~text
FontRef do documento
      ↓
fonte incorporada?
      ↓ não
fonte instalada?
      ↓ não
fallback compatível
~~~

**Font fallback** escolhe outra fonte quando a original não existe ou não contém determinado glifo.

A ausência de fonte nunca deve reescrever automaticamente o `FontRef` original. O documento precisa continuar sabendo qual fonte o autor pediu.

## Font resolution policy

`FontRef` é intenção autoral; `ResolvedFontId` é Derived State.

A resolução segue ordem determinística:

1. fonte incorporada explicitamente referenciada pelo documento;
2. face instalada que corresponda à família/style/axes pedidos;
3. fallback de script/language configurado pelo resolver;
4. fallback genérico da plataforma somente como último recurso.

O resolver produz também um **font fingerprint** derivado — identidade baseada em conteúdo/face/variation usada — para cache e diagnóstico.

~~~text
FontRef
↓
ResolvedFont {
  source,
  face_index,
  content_hash/fingerprint,
  variation_coordinates
}
~~~

O fingerprint não substitui FontRef e não é persistido como verdade autoral.

Quando fallback de plataforma muda entre computadores, o documento continua válido, mas Render/Export produz diagnóstico de substituição. Para output reproduzível, a fonte precisa ser incorporada ou explicitamente empacotada quando a licença permitir.

## Glyph representation

Depois do shaping, um glyph resolvido pode possuir diferentes representações:

~~~text
Outline glyph
COLR/CPAL color glyph
Embedded raster glyph
SVG-in-OpenType glyph
Missing/unsupported glyph
~~~

O Text Engine resolve qual representação está disponível; Render recebe um contrato derivado e não redescobre intenção tipográfica no SceneGraph.

### Outline glyph

Outline normal usa `ttf-parser` para metadata/outlines quando precisamos de vetor e `fontdue` para coverage raster no software renderer.

### COLR/CPAL

Glyph colorido COLR/CPAL é avaliado como paint graph/layers derivados, respeitando palette da fonte e foreground color quando aplicável.

Ele entra no Render Model como primitivas/paint data derivados; não é achatado para bitmap autoral.

### Embedded raster glyph

Quando a fonte oferece bitmap embutido apropriado ao tamanho, o resolver pode expô-lo como recurso raster derivado.

Decode respeita os mesmos limites de segurança de images/resources.

### SVG-in-OpenType

SVG glyph é conteúdo externo incorporado à fonte e portanto input não confiável.

A v0.1 pode:

- usar o pipeline SVG seguro do importer/render adapter quando disponível;
- ou retornar `UnsupportedColorGlyph` e tentar outline fallback da mesma fonte.

Nunca executar scripts, external URLs ou conteúdo ativo de SVG de fonte.

### Fallback final

Se nenhuma representação utilizável existir, o layout preserva advance/cluster quando possível e Render usa missing-glyph/tofu diagnosticável.

Não substituir silenciosamente o caractere por outro code point.

## Frame text

Layout recebe:

- geometria do frame;
- estilos de parágrafo;
- runs shaped;
- regras de coluna;
- baseline grid.

E produz linhas posicionadas.

Decisões relevantes:

- overflow;
- inset/padding;
- vertical alignment;
- columns;
- baseline grid;
- keep-with-next;
- widow/orphan.

### Widow e orphan

Em layout editorial:

- **orphan** é uma linha isolada de um parágrafo deixada no final de uma página/coluna;
- **widow** é uma linha isolada que aparece sozinha no início da próxima página/coluna.

Essas regras são futuras, mas o modelo de layout não deve impedir sua adição.

## Baseline grid

**Baseline** é a linha imaginária sobre a qual as letras se apoiam.

Uma **baseline grid** cria linhas horizontais regulares para alinhar texto de diferentes frames e colunas, comum em diagramação editorial.

O Core guarda a configuração; Layout Engine calcula o alinhamento.

## Text on path

O Core referencia path + offset + orientação.

O Engine calcula o comprimento acumulado da curva e posiciona cada glifo de acordo com:

- posição ao longo do path;
- tangente local;
- baseline;
- orientação escolhida.

A **tangente** é a direção instantânea da curva naquele ponto.

## Text wrap

**Text wrap** faz o texto contornar outro objeto.

Pipeline:

~~~text
objeto
 ↓
contorno/bounds avaliado
 ↓
expandir pelo padding
 ↓
calcular regiões bloqueadas
 ↓
recalcular largura disponível de cada linha
~~~

Isso pertence ao Layout Engine, não ao `TextObject`.

## Linked frames

Frames podem formar uma cadeia:

~~~text
Frame A
  ↓ overflow
Frame B
  ↓
Frame C
~~~

Quando A muda, B e C podem precisar de novo layout.

O Engine deve:

- detectar ciclos;
- invalidar somente frames downstream;
- preservar ordem estável.

## Tables e data merge

Table é modelo de documento. O Engine calcula:

- largura de colunas;
- altura de linhas;
- conteúdo que expande células;
- divisão entre páginas quando suportada.

**Data merge** combina um template com uma fonte de dados para criar várias instâncias, como etiquetas, crachás ou cartões personalizados.

A geração precisa ocorrer por operação explícita e auditável.

## Render

Text Engine entrega glifos posicionados.

Render cuida de:

- rasterização;
- outlines vetoriais;
- antialiasing;
- cache de glifos.

### Glyph atlas

Um **glyph atlas** é uma textura ou superfície grande contendo muitos glifos já rasterizados.

Em vez de rasterizar a mesma letra repetidamente:

~~~text
"A" rasterizado uma vez
       ↓
armazenado no atlas
       ↓
reutilizado em várias posições
~~~

Isso reduz trabalho e chamadas de render.

O atlas é cache derivado. Nunca faz parte do documento.


## Pipeline canônica de parágrafo

Para cada parágrafo:

~~~text
UTF-8 source
↓ validate TextRange boundaries
grapheme / word segmentation
↓
BiDi paragraph resolution
↓
style/script/language runs
↓
font resolution + fallback runs
↓
rustybuzz shaping
↓
line-break opportunities + optional hyphenation
↓
line fitting
↓
justification/alignment
↓
positioned GlyphRuns
~~~

A pipeline pode fazer passes adicionais para fallback, mas a ordem semântica acima é fixa.

## Font fallback

Fallback é resolvido **por cluster**, não trocando a fonte inteira do parágrafo por conveniência.

Direção:

1. tentar a FontRef solicitada;
2. identificar clusters sem cobertura;
3. procurar fallback compatível com script/language;
4. reagrupar somente trechos necessários;
5. shape novamente os trechos afetados;
6. manter a FontRef autoral inalterada.

Fallback precisa preservar cluster boundaries. Não dividir uma sequência combinante entre fontes quando isso produzir shaping inválido.

## Line fitting

A v0.1 usa algoritmo guloso determinístico para quebra de linha:

1. acumular clusters/runs enquanto cabem;
2. guardar a última oportunidade válida de quebra;
3. quando exceder largura, quebrar na última oportunidade;
4. se não houver oportunidade e a política permitir, usar hyphenation;
5. se uma unidade indivisível ainda exceder o frame, marcar overflow.

Algoritmos globais de otimização tipográfica podem ser adicionados depois, mas não são necessários para uma base previsível.

## Justification

Justification distribui espaço somente em pontos permitidos pelo script/layout.

Não implementar “justificar adicionando espaço entre todos os glyphs”.

A v0.1 prioriza:

- espaços expansíveis;
- regras do script;
- tracking somente quando a política permitir;
- nenhum alongamento arbitrário de formas de glifo.

Justificação avançada para scripts específicos pode evoluir sem mudar o modelo autoral.

## Incremental layout

Text layout é derivado e precisa poder ser invalidado por região lógica.

Uma alteração em um parágrafo:

~~~text
paragraph N changed
↓
reshape N
↓
relayout N
↓
se altura/overflow mudou
   invalidate downstream linked frames
~~~

Não relayoutar o documento inteiro por padrão.

## Outlines de texto

Converter texto para curves é Command explícito.

Pipeline:

~~~text
TextObject
↓ resolve fonts
shape
↓ glyph IDs + positions
font outline extraction
↓ transform glyph outlines
VectorPath objects
~~~

`fontdue` não serve para essa etapa porque rasteriza glifos. Outline extraction usa parser/font backend apropriado.

O Command precisa:

- falhar claramente se outline não estiver disponível;
- preservar Appearance onde aplicável;
- gerar ObjectId/ContourId/NodeId novos para a geometria materializada;
- armazenar inverse data para Undo.

## Tables

Tables permanecem **pós-v0.1-stable** no motor editorial. Não introduzir um modelo incompleto no Core apenas para antecipá-las.

O Layout Engine permanece preparado para blocos/frames compostos, mas a especificação de Table terá página própria quando entrar no roadmap de implementação.

## Data merge

Data merge é Engine/automation, não um tipo especial de layout.

Template permanece Document normal. O merge recebe dataset externo validado e produz documentos/instâncias através de Commands/export pipeline.

## Cache keys

Cache de shaping/layout usa pelo menos:

~~~text
text revision/content hash
+ CharacterStyle
+ ParagraphStyle
+ resolved font identity/content hash
+ language/script/direction
+ frame geometry
+ layout policy
~~~

Glyph raster cache não faz parte dessa key; é responsabilidade do Render.

## Erros

Separar:

~~~text
MissingRequestedFont → degraded com fallback
MissingGlyph         → degraded/tofu conforme policy
InvalidTextRange     → domain/command error
ShapingFailure       → engine error
LayoutOverflow       → estado de layout, não erro fatal
CyclicTextFlow       → rejeitado pelo Core
~~~

Overflow é resultado válido e consultável.

## Invariantes

1. Unicode source e estilos são autorais; glyph IDs/positions são derivados.
2. Grapheme, BiDi, shaping e line breaking permanecem etapas distintas.
3. `rustybuzz` é o shaper OpenType inicial.
4. Fallback acontece por clusters e nunca reescreve FontRef.
5. Hyphenation é serviço opcional por idioma.
6. A v0.1 usa line fitting guloso, determinístico e testável.
7. Layout incremental invalida somente texto/frames downstream necessários.
8. Convert to Curves é materialização explícita e usa outlines, não raster glyphs.
9. Glyph atlas pertence ao Render.
10. Tables não entram com modelo incompleto antes de sua especificação.
11. Grapheme/BiDi/line-break seguem UAX #29/#9/#14, respectivamente.
12. Tags de idioma seguem BCP 47.
13. Texto autoral não é normalizado Unicode silenciosamente.

## Evaluation materializada em 2026-10-10

Fontes são carregadas explicitamente; o registry retém bytes compartilhados e a compilação não depende de reler o arquivo original. Layout resolve styles, fallback por grapheme, BiDi conectado, shaping, wrap/hyphenation e outlines. O compiler materializa texto como geometria de render.

Text-on-path usa arclength, tangente, start offset e transformação entre path e texto, preservando o conteúdo autoral. Linked frames concatenam o conteúdo em ordem, sem inserir separadores, e distribuem linhas inteiras entre frames; não reescrevem as strings do Core. A avaliação limita a cadeia a 64 frames e 65.536 bytes. Overflow e condições sem suporte são diagnosticados.

Evidência: `text_raster_contracts.rs`, `text_path_regressions.rs` (3 testes) e testes de layout. Baseline grids, layout editorial completo, caret e seleção tipográfica continuam pendentes. Gates em [Verification](#/docs/00-architecture/verification.md).
