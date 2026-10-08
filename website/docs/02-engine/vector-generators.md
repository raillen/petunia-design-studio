# Vector Generators

Vector Generators produzem geometria determinística a partir de dados estruturados.

A v0.1 define dois generators iniciais:

~~~text
QR Code
Barcode
~~~

Eles pertencem ao Engine. Core persiste `GeneratorSpec`. Render recebe VectorPath derivado.

## Por que não gerar paths imediatamente

Se o usuário cria um QR com payload:

~~~text
https://example.com
~~~

o dado autoral importante é o payload + parâmetros do código.

Guardar apenas milhares de rectangles perderia essa intenção.

Portanto:

~~~text
GeneratorSpec
↓ evaluate
VectorPath derived
↓
Appearance
↓
Render
~~~

`Convert to Curves` materializa a geometria quando desejado.

## Core model

O modelo autoral canônico — `GeneratedVectorObject`, `GeneratorSpec`, `QrCodeSpec` e `BarcodeSpec` — pertence ao Core e está documentado em [conteúdo vetorial gerado](#/docs/01-core/generated-content.md).

O Engine recebe esses specs, valida regras do standard que exigem algoritmo especializado e produz `VectorPath` derivado.

GeneratedVector é `SceneItem` próprio e não pertence a `ParametricShape`: QR/Barcode não são primitivas geométricas do mesmo domínio de Rectangle/Ellipse/Star.

## QR Code

QR segue o padrão ISO/IEC 18004 através de encoder especializado e testado.

Não reimplementar Reed-Solomon, data placement e mask scoring casualmente só para evitar uma dependência pequena.

O encoder concreto fica atrás de:

~~~rust
pub trait QrEncoder {
    fn encode(
        &self,
        spec: &QrCodeSpec,
    ) -> Result<QrMatrix, QrError>;
}
~~~

A crate concreta é escolhida no milestone após revisão de manutenção/licença/test vectors; o Core não depende dela.

## QrCodeSpec

Direção:

~~~rust
pub struct QrCodeSpec {
    pub payload: QrPayload,
    pub error_correction: QrErrorCorrection,
    pub version: QrVersionPolicy,
    pub mask: QrMaskPolicy,
    pub quiet_zone_modules: u8,
}

pub enum QrPayload {
    Text(String),
    Bytes(Vec<u8>),
}
~~~

### Error correction

~~~text
L
M
Q
H
~~~

são níveis persistentes conforme o padrão QR.

Não serializar nomes localizados.

### Version

~~~text
Auto
Fixed(version)
~~~

Auto escolhe a menor versão que comporta payload + ECC.

Fixed falha se os dados não couberem; não muda versão silenciosamente.

### Mask

~~~text
Auto
Fixed(mask_id)
~~~

Auto usa scoring definido pelo padrão.

Fixed existe para interoperabilidade/testes avançados.

### Quiet zone

Default técnico: 4 modules.

Valor abaixo do mínimo de interoperabilidade definido pela policy pode ser rejeitado ou marcado como non-standard; a v0.1 não reduz automaticamente quiet zone para “caber”.

## QR evaluation

Encoder produz matriz:

~~~text
QrMatrix
width × height
bool dark/light
~~~

A matriz é Derived State.

Engine converte módulos dark em geometria vetorial.

## Module coalescing

Não criar um Rectangle SceneObject por módulo.

Isso produziria milhares de objetos e poluiria o SceneGraph.

Em vez disso, gerar um único VectorPath/compound geometry.

Otimização:

1. encontrar runs horizontais contíguos de dark modules;
2. mesclar runs idênticos em rows consecutivas;
3. produzir rectangles máximos;
4. reunir em contours/path de fill NonZero/EvenOdd apropriado.

A otimização não altera a matriz.

~~~text
████
████
~~~

vira um rectangle 4×2, não oito rectangles 1×1.

## Coordinates

QR local space usa module = 1 document unit lógico antes do object transform.

~~~text
(0,0)
↓
matrix + quiet zone
↓
local bounds
~~~

SceneNode transform/size controla escala final.

Nunca gerar modules em pixels de tela.

Scaling não pode distorcer X/Y de forma diferente se o objetivo é QR válido. Engine expõe preferred aspect ratio 1:1; ferramenta futura decide constraints, mas validator/export pode warning sobre non-uniform transform.

## Appearance

Generator produz apenas geometry.

Color vem de Appearance.

Para máxima interoperabilidade, export validator pode alertar contraste baixo, mas generator não força preto/branco no Document.

## Barcode

Barcode inicial suporta:

~~~text
Code 128
EAN-13
UPC-A
~~~

Adicionar symbology nova exige spec/validator próprio quando as regras diferirem.

## BarcodeSpec

~~~rust
pub struct BarcodeSpec {
    pub symbology: BarcodeSymbology,
    pub data: String,
    pub quiet_zone_modules: f64,
    pub bar_height_modules: f64,
}
~~~

Human-readable label não faz parte do geometry generator v0.1.

A ferramenta futura pode criar TextObject separado, preservando tipografia normal do Petunia.

## Code 128

Code 128 suporta amplo conjunto de caracteres através de code sets.

O encoder especializado decide code-set switching de forma determinística para compactar quando possível.

Pipeline conceitual:

~~~text
input data
↓ encode symbols
↓ start/data/checksum/stop
↓ module pattern
↓ vector bars
~~~

Checksum é calculado pelo encoder; user não digita checksum manual como parte separada do spec.

## EAN-13

Input autoral representa 12 ou 13 dígitos conforme policy.

- 12 digits → calcular check digit;
- 13 digits → validar check digit.

Caracter inválido produz erro.

Não “corrigir” número silenciosamente.

## UPC-A

Segue política equivalente de validação/check digit.

## Bars to VectorPath

Assim como QR:

- não criar SceneNode por bar;
- gerar compound VectorPath derivado;
- mesclar rectangles somente quando isso não altera module widths/guard patterns.

Units permanecem exatas em module space.

## Dependency policy

QR/Barcode standards possuem muitos detalhes de encoding/checksum/error correction.

A filosofia do Petunia favorece uma biblioteca Rust focada, pequena e testada atrás de adapter.

Critérios:

- implementação do standard completa;
- licença compatível;
- test vectors;
- sem framework gráfico embutido;
- acesso à matrix/module pattern, não apenas PNG/SVG string;
- determinismo.

Não escolher uma biblioteca que só gere bitmap, pois perderíamos controle do vetor.

## Live update

Editar payload/params:

~~~text
GeneratorSpec changed
↓
invalidate generated geometry
↓
re-evaluate
↓
Render
~~~

ObjectId permanece o mesmo.

## Convert to Curves

Command:

1. avaliar generator;
2. materializar VectorPath;
3. criar ContourId/NodeId persistentes;
4. substituir GeneratedVector por PathObject;
5. preservar ObjectId;
6. preservar Appearance;
7. guardar GeneratorSpec na inverse data para Undo.

## Errors

~~~text
PayloadTooLarge
InvalidCharacter
InvalidChecksum
UnsupportedVersion
UnsupportedSymbology
InvalidQuietZone
EncoderFailure
~~~

Erro não produz QR/barcode parcial apresentado como válido.

## Testing

Usar test vectors oficiais/confiáveis para matrix/module pattern.

Além disso, rasterizar output com software renderer e tentar decodificar com decoder independente em testes de integração.

Casos:

- payload mínimo;
- limite de versão;
- Unicode/text encoding;
- cada ECC;
- fixed mask;
- EAN valid/invalid checksum;
- extreme scaling;
- quiet zone;
- deterministic output.

## Invariantes

1. QR/Barcode persistem data + params, não milhares de SceneNodes.
2. GeneratedVector é conceito separado de ParametricShape.
3. Encoding standard usa backend especializado atrás de adapter.
4. Geometry output é um compound VectorPath derivado.
5. QR Auto Version escolhe menor versão compatível.
6. Fixed Version nunca é alterada silenciosamente.
7. Quiet zone é parâmetro explícito.
8. Barcode valida charset/checksum por symbology.
9. Human-readable text usa Text system, não outline improvisado no generator.
10. Convert to Curves preserva ObjectId e Appearance.
11. Mesmo spec produz o mesmo module pattern.
12. Erro nunca produz código parcialmente válido.
