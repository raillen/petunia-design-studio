import type { UserConfig } from 'vitepress'

// Shared (language-agnostic) VitePress configuration for the
// Petunia Design Studio living-documentation site.
//
// Everything in this file applies to ALL locales. Language-specific
// navigation, sidebar and UI strings live in `./locales/<lang>.ts`.
export const shared: UserConfig = {
  title: 'Petunia Design Studio',
  description: 'Canonical living documentation: vector + photo design, non-destructive editing, and open automation.',

  // Strict routing rules (see bible/SPEC-001).
  cleanUrls: true,
  ignoreDeadLinks: false,
  sitemap: {
    hostname: 'https://petuniadesign.vercel.app',
    // 404 pages are routing fallbacks, not content: keep them out of SEO.
    transformItems: (items) => items.filter((item) => !item.url.includes('404'))
  },
  // NOTE: `lastUpdated` stays OFF on purpose. Enabling it makes the default
  // theme import dayjs (UMD, no `default` export), which crashes `vitepress
  // dev` under pnpm with "does not provide an export named 'default'" and
  // breaks hydration. Revisit after a VitePress/Vite upgrade; per-locale
  // `lastUpdated.text` labels are already in place for that day.

  // Legacy Notion-export pages kept at docs/ root are NOT part of the
  // public site. They stay in git for archaeology but are excluded from
  // routing, sidebar generation and dead-link checking.
  // NOTE: the pattern requires TWO leading digits + space so it never
  // matches VitePress-reserved pages like 404.md.
  srcExclude: ['[0-9][0-9] *.md', 'Aubrieta Design *.md', 'PRUMO.md'],

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }],
    ['meta', { name: 'theme-color', content: '#b048b5' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:site_name', content: 'Petunia Design Studio Docs' }]
  ],

  markdown: {
    lineNumbers: true,
    image: {
      lazyLoading: true
    }
  },

  vite: {
    // Dev-server hardening (no effect on production builds): mermaid
    // depends on dayjs, which ships UMD (`dayjs.min.js`) with no ESM
    // `default` export. Without forced pre-bundling, `vitepress dev`
    // serves that file raw and the browser throws "does not provide an
    // export named 'default'", breaking hydration. esbuild interop
    // generates a proper default export for both entries.
    optimizeDeps: {
      include: ['mermaid', 'dayjs']
    }
  },

  // Mermaid diagrams are rendered client-side by vitepress-plugin-mermaid
  // (wired in config.mts via withMermaid). Keep node IDs stable across
  // locales; only translate the visible labels.
  mermaid: {
    theme: 'default',
    securityLevel: 'strict'
  } as UserConfig['mermaid'],

  themeConfig: {
    logo: '/logo.svg',
    socialLinks: [
      { icon: 'github', link: 'https://github.com/raillen/petunia-design-studio' }
    ]
  }
}
