# Color Management Engine

`palette` ajuda em matemática/conversões simples, mas não substitui um CMM ICC profissional.

## CMM

Para ICC, a opção Rust madura a avaliar é `lcms2`, wrapper de Little CMS. O domínio deve escondê-lo atrás de trait para permitir testes e troca futura.

```rust
pub trait ColorManagementEngine {
    fn build_transform(&self, spec: ColorTransformSpec) -> Result<ColorTransformId>;
    fn convert_pixels(&self, transform: ColorTransformId, src: PixelView, dst: PixelViewMut) -> Result<()>;
    fn convert_color(&self, transform: ColorTransformId, color: ColorValue) -> Result<ColorValue>;
}
```

## Assign versus Convert

**Assign** troca a interpretação/perfil, mantendo números. **Convert** calcula novos números para preservar aparência entre espaços. Devem ser Commands diferentes.

## Working space

Documento define working profile. Recursos importados têm profile de origem. Política de import decide preservar perfil ou converter para working space.

## Rendering intents

Suportar os quatro intents ICC:
- absolute colorimetric
- relative colorimetric
- perceptual
- saturation.

Black point compensation é configuração separada.

## Soft proof

Soft proof é transformação de visualização/adjustment e **não modifica fonte**. Pode haver múltiplos proof presets.

## Linearização

Pipeline de composição precisa trabalhar em linear light quando a operação exigir. O Color Engine fornece transform encoded ↔ linear; Render decide onde aplicar.

## LUT cache

ICC transforms podem ser caros. Cache por:
- source profile hash
- destination profile hash
- intent
- BPC
- pixel format.

## Spot colors

Tela usa alternate color para preview, mas export/prepress mantém separação spot quando o formato permite.

## Gamut warning

Engine calcula out-of-gamut mask; Render colore overlay. Não alterar pixels.

## HDR

Não fechar arquitetura em 0…1. Perfis/transfer functions futuras podem incluir HDR; tipos internos devem aceitar valores float estendidos e metadata apropriada.
