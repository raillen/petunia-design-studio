import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'
import { shared } from './shared'
import { enConfig } from './locales/en'
import { ptConfig } from './locales/pt'

// VitePress entry point with Mermaid support and modular i18n.
//
// To add a new language (e.g. Spanish):
//   1. Create `docs/.vitepress/locales/<lang>.ts` exporting `<lang>Config`
//      (copy `pt.ts` as a template).
//   2. Import it here and register it under `locales`.
//   3. Mirror the English page tree under `docs/<lang>/`
//      (see `contributing/translations` and `scripts/check-i18n.js`).
export default withMermaid(
  defineConfig({
    ...shared,
    locales: {
      root: enConfig,
      pt: ptConfig
    }
  })
)
