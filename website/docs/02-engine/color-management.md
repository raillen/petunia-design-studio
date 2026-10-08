# Color Management Engine

`palette` ajuda em matemática e conversões simples de cor, mas não substitui um sistema profissional de gerenciamento ICC.

## CMM

**CMM** significa *Color Management Module*.

É o componente que calcula transformações de cor entre perfis diferentes.

Exemplo:

~~~text
RGB da câmera
   ↓ perfil de origem
CMM
   ↓ perfil de destino
CMYK da impressora
~~~

Para ICC, a opção Rust madura a avaliar é `lcms2`, wrapper de Little CMS. O domínio deve escondê-lo atrás de trait para permitir testes e troca futura.

~~~rust
pub trait ColorManagementEngine {
    fn build_transform(
        &self,
        spec: ColorTransformSpec,
    ) -> Result<ColorTransformId>;

    fn convert_pixels(
        &self,
        transform: ColorTransformId,
        src: PixelView,
        dst: PixelViewMut,
    ) -> Result<()>;

    fn convert_color(
        &self,
        transform: ColorTransformId,
        color: ColorValue,
    ) -> Result<ColorValue>;
}
~~~

## ICC profile

**ICC profile** descreve como os valores numéricos de um dispositivo ou espaço correspondem a cores reais.

Os valores:

~~~text
R=200 G=40 B=30
~~~

não são uma cor completamente definida sem sabermos o espaço/perfil usado.

O perfil fornece essa interpretação.

## PCS

Perfis ICC usam um espaço intermediário chamado **PCS — Profile Connection Space**.

Conceitualmente:

~~~text
perfil A
  ↓
PCS
  ↓
perfil B
~~~

Isso evita precisar de uma conversão específica para cada par possível de dispositivos.

## Assign versus Convert

**Assign profile** troca a interpretação dos mesmos números.

~~~text
números RGB permanecem iguais
perfil muda
aparência pode mudar
~~~

**Convert profile** recalcula os números para tentar preservar a aparência.

~~~text
cor aparente desejada permanece
valores RGB/CMYK podem mudar
~~~

Por isso são Commands diferentes.

## Transform contract

Uma transformação de cor é identificada por suas entradas semânticas, não por um ponteiro de backend.

~~~text
source profile/content hash
+
destination profile/content hash
+
pixel/color format
+
rendering intent
+
BPC
+
flags semânticos
→ ColorTransformKey
~~~

Transformações são cache derivado. Alterar perfil ou intent produz outra key; nunca modifica valores autorais sem Command de Convert.

Para cores escalares, o Engine pode usar a mesma transformação conceitual utilizada para pixels, evitando duas matemáticas divergentes.

## Working space

O **working space** é o espaço de cor principal no qual o documento trabalha.

Recursos importados podem:

- manter perfil próprio;
- ser convertidos para o working space;
- pedir decisão do usuário conforme a política definida.

Essa decisão deve ser explícita.

## Rendering intents

Um perfil pode encontrar cores que o destino não consegue reproduzir. O **rendering intent** define a estratégia usada nessa situação.

Os quatro intents ICC são:

### Relative colorimetric

Cores que cabem no destino são preservadas o máximo possível. Cores fora do gamut são levadas ao limite reproduzível.

O branco do espaço de origem é adaptado ao branco do destino.

É comum em impressão quando fidelidade local é importante.

### Absolute colorimetric

Preserva também a relação com o branco do espaço de origem.

É útil principalmente para simular a aparência de outro papel/dispositivo em soft proof.

### Perceptual

Pode comprimir várias cores para preservar relações visuais gerais quando o gamut do destino é menor.

É útil para imagens com muitas cores fora do gamut, embora o comportamento concreto dependa do perfil/CMM.

### Saturation

Prioriza intensidade/saturação em vez de fidelidade colorimétrica.

É mais comum em gráficos/apresentações do que em fotografia.

## Black point compensation

**Black Point Compensation — BPC** adapta a faixa de sombras quando o preto mais escuro da origem e do destino são diferentes.

Sem compensação, detalhes escuros podem ser comprimidos de forma abrupta.

BPC é configuração separada do rendering intent.

## Soft proof

**Soft proof** simula na tela como o trabalho tende a aparecer em outro dispositivo ou condição de impressão.

~~~text
documento
  ↓
perfil da impressão simulada
  ↓
perfil do monitor
  ↓
preview
~~~

Soft proof não modifica as cores autorais do documento.

## Encoded versus linear light

Espaços como sRGB normalmente armazenam valores **encoded**, ajustados por uma curva de transferência para distribuição eficiente e visualização.

Muitas operações de luz precisam de valores **lineares**, nos quais relações numéricas correspondem melhor à intensidade luminosa.

~~~text
encoded RGB
   ↓ linearização
linear RGB
   ↓ cálculo/composição
encoded RGB para saída
~~~

O Color Engine fornece as transformações; Render declara onde elas são necessárias.

## LUT

**LUT** significa *Look-Up Table*.

É uma tabela pré-calculada usada para substituir cálculos repetitivos por consultas rápidas.

Exemplo conceitual:

~~~text
entrada 0.00 → saída ...
entrada 0.01 → saída ...
entrada 0.02 → saída ...
...
~~~

Transformações ICC podem ser caras. O Engine pode cachear transformações/LUTs por:

- hash do perfil de origem;
- hash do perfil de destino;
- rendering intent;
- BPC;
- formato de pixel.

A LUT é cache derivado e reconstruível.

## Spot colors

**Spot color** representa uma tinta específica de impressão, não apenas uma cor CMYK aproximada.

Na tela usamos uma cor alternativa para preview. Na exportação compatível, a identidade da tinta precisa permanecer preservada.

## Gamut

**Gamut** é o conjunto de cores que um espaço ou dispositivo consegue representar.

Uma cor pode existir no working space e ficar fora do gamut de uma impressora.

## Gamut warning

O Engine pode gerar uma máscara das regiões fora do gamut.

~~~text
documento
   ↓
teste contra destino
   ↓
máscara out-of-gamut
   ↓
overlay visual
~~~

Render mostra o aviso; o documento não é alterado.

## HDR

**HDR — High Dynamic Range** representa faixas de luminância mais amplas do que pipelines tradicionais.

Por isso a arquitetura não deve assumir que todos os canais ficam permanentemente entre 0 e 1.

Valores float intermediários podem ultrapassar esse intervalo antes da etapa de output/tone mapping.

A política HDR completa fica aberta até existir caso de uso concreto.


## Pipeline definido

Para conteúdo RGB comum:

~~~text
encoded source values
↓ source profile / transfer function
managed linear working representation quando a operação exigir luz linear
↓ effects/compositing
output transform
↓ display/export profile
~~~

Não existe regra “converter tudo para sRGB”. O espaço de trabalho e o destino determinam a transformação.

Para CMYK autoral, preservar os canais/perfil enquanto a operação não exigir conversão para outro espaço. Render de tela pode usar uma representação RGB derivada; isso não reescreve CMYK do Document.

## Política de perfis ausentes ou inválidos

- perfil incorporado válido → usar;
- perfil referenciado ausente → manter referência e marcar estado unresolved;
- arquivo RGB sem perfil → aplicar a política explícita de import, nunca adivinhar silenciosamente depois;
- perfil corrompido/inseguro → rejeitar transformação e produzir diagnóstico;
- soft proof sem perfil de destino resolvido → indisponível, sem alterar o Document.

## Threading

Objetos de transformação do CMM podem ser caros. O cache precisa respeitar a segurança de thread do backend utilizado.

A API Petunia não expõe handles mutáveis do CMM para workers. Se o backend exigir handles por thread, o adapter cria/cacheia instâncias adequadas internamente.

## Invariantes

1. Little CMS 2 é o backend ICC inicial, encapsulado atrás da API Petunia.
2. Tipos/handles do CMM nunca entram no Core ou PTND.
3. Assign e Convert continuam Commands semanticamente distintos.
4. Soft proof é View State e nunca reescreve cores autorais.
5. Transform cache é derivado e chaveado por perfis + parâmetros semânticos.
6. Conteúdo sem perfil segue política explícita de import; não é reinterpretado silenciosamente.
7. CMYK/Spot autoral é preservado sempre que a operação não exige materialização/conversão.
8. Render/output usa transformações derivadas; não altera o Document.
