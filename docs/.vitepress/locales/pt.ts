import type { DefaultTheme } from 'vitepress'

// Portuguese (pt-BR) locale configuration.
// Served under the `/pt/` prefix. Every canonical English page must have
// a 1:1 mirror here (enforced by scripts/check-i18n.js).
export const ptConfig = {
  label: 'Português',
  lang: 'pt-BR',
  link: '/pt/',
  title: 'Petunia Design Studio',
  description: 'Documentação viva canônica: design vetorial + fotográfico, edição não destrutiva e automação aberta.',

  themeConfig: {
    nav: [
      { text: 'Guia', link: '/pt/getting-started/' },
      { text: 'Manual', link: '/pt/manual/' },
      { text: 'Ferramentas', link: '/pt/tools/' },
      { text: 'Desenvolvedores', link: '/pt/developers/' },
      { text: 'Bíblia', link: '/pt/bible/' }
    ],

    sidebar: [
      {
        text: 'Primeiros passos',
        items: [
          { text: 'Visão geral', link: '/pt/getting-started/' },
          { text: 'Instalação', link: '/pt/getting-started/installation' },
          { text: 'Início rápido (15 min)', link: '/pt/getting-started/quickstart' }
        ]
      },
      {
        text: 'Manual',
        items: [
          { text: 'Visão geral', link: '/pt/manual/' },
          { text: 'Glossário', link: '/pt/glossary' }
        ]
      },
      {
        text: 'Ferramentas',
        items: [{ text: 'Catálogo de ferramentas', link: '/pt/tools/' }]
      },
      {
        text: 'Desenvolvedores',
        items: [
          { text: 'Visão geral', link: '/pt/developers/' },
          { text: 'Arquitetura', link: '/pt/developers/architecture' },
          { text: 'Compilar e testar', link: '/pt/developers/build-test' },
          { text: 'ADRs', link: '/pt/developers/adr/' },
          { text: 'ADR-001: EffectChain', link: '/pt/developers/adr/ADR-001-effect-chain' }
        ]
      },
      {
        text: 'Contribuição',
        items: [
          { text: 'Diretrizes', link: '/pt/contributing/guidelines' },
          { text: 'Traduções', link: '/pt/contributing/translations' }
        ]
      },
      {
        text: 'Bíblia (SSOT)',
        items: [
          { text: 'Constituição', link: '/pt/bible/' },
          { text: 'SPEC-001: Documentação viva', link: '/pt/bible/SPEC-001' }
        ]
      }
    ] as DefaultTheme.Sidebar,

    editLink: {
      pattern: 'https://github.com/raillen/petunia-design-studio/edit/main/docs/:path',
      text: 'Editar esta página no GitHub'
    },

    footer: {
      message: 'Publicado sob MIT OR Apache-2.0.',
      copyright: 'Copyright © 2026 Colaboradores do Petunia Design Studio'
    },

    docFooter: {
      prev: 'Página anterior',
      next: 'Próxima página'
    },

    outline: {
      label: 'Nesta página'
    },

    lastUpdated: {
      text: 'Última atualização'
    },

    langMenuLabel: 'Trocar idioma',
    returnToTopLabel: 'Voltar ao topo',
    sidebarMenuLabel: 'Menu',
    darkModeSwitchLabel: 'Aparência',
    lightModeSwitchTitle: 'Mudar para o tema claro',
    darkModeSwitchTitle: 'Mudar para o tema escuro',

    search: {
      provider: 'local',
      options: {
        locales: {
          pt: {
            translations: {
              button: {
                buttonText: 'Buscar na documentação',
                buttonAriaLabel: 'Buscar na documentação'
              },
              modal: {
                noResultsText: 'Sem resultados para',
                resetButtonTitle: 'Limpar busca',
                footer: {
                  selectText: 'para selecionar',
                  navigateText: 'para navegar',
                  closeText: 'para fechar'
                }
              }
            }
          }
        }
      }
    }
  }
}
