import type { DefaultTheme } from 'vitepress'

// English (canonical, default) locale configuration.
// Served at the site root: `/`.
export const enConfig = {
  label: 'English',
  lang: 'en-US',
  link: '/',
  title: 'Petunia Design Studio',
  description: 'Canonical living documentation: vector + photo design, non-destructive editing, and open automation.',

  themeConfig: {
    nav: [
      { text: 'Guide', link: '/getting-started/' },
      { text: 'Manual', link: '/manual/' },
      { text: 'Tools', link: '/tools/' },
      { text: 'Developers', link: '/developers/' },
      { text: 'Bible', link: '/bible/' }
    ],

    sidebar: [
      {
        text: 'Getting started',
        items: [
          { text: 'Overview', link: '/getting-started/' },
          { text: 'Installation', link: '/getting-started/installation' },
          { text: 'Quickstart (15 min)', link: '/getting-started/quickstart' }
        ]
      },
      {
        text: 'Manual',
        items: [
          { text: 'Overview', link: '/manual/' },
          { text: 'Glossary', link: '/glossary' }
        ]
      },
      {
        text: 'Tools',
        items: [{ text: 'Tool catalog', link: '/tools/' }]
      },
      {
        text: 'Developers',
        items: [
          { text: 'Overview', link: '/developers/' },
          { text: 'Architecture', link: '/developers/architecture' },
          { text: 'Build & test', link: '/developers/build-test' },
          { text: 'ADRs', link: '/developers/adr/' },
          { text: 'ADR-001: EffectChain', link: '/developers/adr/ADR-001-effect-chain' },
          { text: 'Studio UI/UX', link: '/developers/uiux-studio' }
        ]
      },
      {
        text: 'Contributing',
        items: [
          { text: 'Guidelines', link: '/contributing/guidelines' },
          { text: 'Translations', link: '/contributing/translations' }
        ]
      },
      {
        text: 'Bible (SSOT)',
        items: [
          { text: 'Constitution', link: '/bible/' },
          { text: 'SPEC-001: Living documentation', link: '/bible/SPEC-001' }
        ]
      }
    ] as DefaultTheme.Sidebar,

    editLink: {
      pattern: 'https://github.com/raillen/petunia-design-studio/edit/main/docs/:path',
      text: 'Edit this page on GitHub'
    },

    footer: {
      message: 'Released under MIT OR Apache-2.0.',
      copyright: 'Copyright © 2026 Petunia Design Studio contributors'
    },

    docFooter: {
      prev: 'Previous page',
      next: 'Next page'
    },

    outline: {
      label: 'On this page'
    },

    lastUpdated: {
      text: 'Last updated'
    },

    langMenuLabel: 'Change language',
    returnToTopLabel: 'Return to top',
    sidebarMenuLabel: 'Menu',
    darkModeSwitchLabel: 'Appearance',
    lightModeSwitchTitle: 'Switch to light theme',
    darkModeSwitchTitle: 'Switch to dark theme',

    search: {
      provider: 'local',
      options: {
        locales: {
          root: {
            translations: {
              button: {
                buttonText: 'Search docs',
                buttonAriaLabel: 'Search docs'
              },
              modal: {
                noResultsText: 'No results for',
                resetButtonTitle: 'Clear search',
                footer: {
                  selectText: 'to select',
                  navigateText: 'to navigate',
                  closeText: 'to close'
                }
              }
            }
          }
        }
      }
    }
  }
}
