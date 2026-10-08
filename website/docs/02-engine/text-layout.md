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
- `unicode-segmentation` para grapheme/word boundaries;
- `unicode-bidi` para o Unicode Bidirectional Algorithm;
- `unicode-linebreak` ou implementação equivalente estritamente baseada em UAX #14 para oportunidades de quebra;
- parser de fonte dedicado como `ttf-parser` para metadata/outlines quando necessário;
- `fontdue` apenas para rasterização de cobertura de glifos, não para shaping nem extração autoral de outlines.

Essas bibliotecas ficam atrás de tipos Petunia. IDs de glifo, structs de fonte e buffers externos não entram no Core.

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

A v0.1 trata hyphenation como serviço opcional por idioma atrás de `HyphenationProvider`.

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
