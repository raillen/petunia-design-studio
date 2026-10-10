# Ícones embarcados

Os SVGs são ativos originais MIT do Phosphor e do Tabler. As revisões Git,
URLs individuais e SHA-256 estão em `manifest.json`; os avisos originais
estão em `phosphor-LICENSE` e `tabler-LICENSE`. O aplicativo funciona sem
buscar ícones pela rede.

Cada ação possui uma chave semântica. A troca de família mantém a mesma
ação. Quando uma variante preenchida não existe no upstream, o catálogo
registra explicitamente o fallback para o contorno da mesma família.
O aplicativo substitui `currentColor` somente por tokens da sua paleta.

`catalog.rs` incorpora os SVGs no binário; `icons.rs` resolve chave, família,
variante e cor. Para auditar integridade: `python3 scripts/verify-icons.py`.
