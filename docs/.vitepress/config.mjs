const enNav = [
  { text: 'Guide', link: '/guide/' },
  { text: 'Examples', link: '/examples/' },
  { text: 'CLI', link: '/reference/cli' },
  { text: 'Architecture', link: '/architecture' },
  { text: 'Advanced', link: '/advanced/' },
  { text: 'Maintainer', link: '/maintainer/' }
]

const zhNav = [
  { text: '入门', link: '/zh/guide/' },
  { text: '示例', link: '/zh/examples/' },
  { text: 'CLI', link: '/zh/reference/cli' },
  { text: '架构', link: '/zh/architecture' },
  { text: '高级', link: '/zh/advanced/' },
  { text: '维护者', link: '/zh/maintainer/' }
]

const enSidebar = {
  '/guide/': [
    {
      text: 'Guide',
      items: [
        { text: 'Overview', link: '/guide/' },
        { text: 'Quickstart', link: '/quickstart' },
        { text: 'Task Recipes', link: '/guide/task-recipes' },
        { text: 'Install', link: '/guide/install' },
        { text: 'Upgrade', link: '/guide/upgrade' },
        { text: 'Troubleshooting', link: '/guide/troubleshooting' }
      ]
    }
  ],
  '/reference/': [
    {
      text: 'Reference',
      items: [
        { text: 'Overview', link: '/reference/' },
        { text: 'CLI Reference', link: '/reference/cli' },
        { text: 'Skills Guide', link: '/reference/skills' },
        { text: 'Conventions', link: '/conventions' }
      ]
    }
  ],
  '/examples/': [
    {
      text: 'Examples',
      items: [
        { text: 'Overview', link: '/examples/' },
        { text: 'Paper Type Playbooks', link: '/examples/paper-type-playbooks' }
      ]
    }
  ],
  '/advanced/': [
    {
      text: 'Advanced',
      items: [
        { text: 'Overview', link: '/advanced/' },
        { text: 'Extend Research Skills', link: '/advanced/extend-research-skills' },
        { text: 'Agent + Skill Collaboration', link: '/advanced/agent-skill-collaboration' },
        { text: 'MCP Providers Setup', link: '/advanced/mcp-providers-setup' },
        { text: 'Zotero Integration', link: '/advanced/mcp-zotero-integration' },
        { text: 'Publish to PyPI', link: '/advanced/publish-pypi' }
      ]
    }
  ],
  '/maintainer/': [
    {
      text: 'Maintainer',
      items: [
        { text: 'Overview', link: '/maintainer/' },
        { text: 'CLAUDE Guide Summary', link: '/maintainer/claude-overview' },
        { text: 'Architecture', link: '/architecture' },
        { text: 'Conventions', link: '/conventions' },
        { text: 'Publish to PyPI', link: '/advanced/publish-pypi' }
      ]
    }
  ]
}

const zhSidebar = {
  '/zh/guide/': [
    {
      text: '入门',
      items: [
        { text: '总览', link: '/zh/guide/' },
        { text: '快速开始', link: '/zh/quickstart' },
        { text: '任务场景', link: '/zh/guide/task-recipes' },
        { text: '安装', link: '/zh/guide/install' },
        { text: '升级', link: '/zh/guide/upgrade' },
        { text: '故障排除', link: '/zh/guide/troubleshooting' }
      ]
    }
  ],
  '/zh/reference/': [
    {
      text: '参考',
      items: [
        { text: '总览', link: '/zh/reference/' },
        { text: 'CLI 参考', link: '/zh/reference/cli' },
        { text: 'Skills 指南', link: '/zh/reference/skills' },
        { text: '规范约定', link: '/zh/conventions' }
      ]
    }
  ],
  '/zh/examples/': [
    {
      text: '示例',
      items: [
        { text: '总览', link: '/zh/examples/' },
        { text: 'Paper Type 路线图', link: '/zh/examples/paper-type-playbooks' }
      ]
    }
  ],
  '/zh/advanced/': [
    {
      text: '高级',
      items: [
        { text: '总览', link: '/zh/advanced/' },
        { text: '扩展 Research Skills', link: '/zh/advanced/extend-research-skills' },
        { text: 'Agent + Skill 协同', link: '/zh/advanced/agent-skill-collaboration' },
        { text: 'MCP Providers 接入', link: '/zh/advanced/mcp-providers-setup' },
        { text: 'Zotero 集成', link: '/zh/advanced/mcp-zotero-integration' },
        { text: '发布到 PyPI', link: '/zh/advanced/publish-pypi' }
      ]
    }
  ],
  '/zh/maintainer/': [
    {
      text: '维护者',
      items: [
        { text: '总览', link: '/zh/maintainer/' },
        { text: 'CLAUDE 指南摘要', link: '/zh/maintainer/claude-overview' },
        { text: '系统架构', link: '/zh/architecture' },
        { text: '规范约定', link: '/zh/conventions' },
        { text: 'PyPI 发布', link: '/zh/advanced/publish-pypi' }
      ]
    }
  ]
}

const commonHead = [
  ['meta', { name: 'theme-color', content: '#0f766e' }],
  ['meta', { name: 'author', content: 'Jiaxin Peng' }]
]

/** @type {import('vitepress').UserConfig} */
export default {
  title: 'Research Skills',
  description: 'Contract-driven academic research workflow documentation for Codex, Claude Code, and Gemini.',
  cleanUrls: true,
  lastUpdated: true,
  head: commonHead,
  ignoreDeadLinks: [/^https?:\/\//],
  locales: {
    root: {
      label: 'English',
      lang: 'en-GB',
      themeConfig: {
        nav: enNav,
        sidebar: enSidebar,
        search: { provider: 'local' },
        outline: { level: [2, 3] },
        socialLinks: [{ icon: 'github', link: 'https://github.com/jxpeng98/research-skills' }],
        footer: {
          message: 'Research Skills documentation',
          copyright: 'MIT License'
        }
      }
    },
    zh: {
      label: '简体中文',
      lang: 'zh-CN',
      link: '/zh/',
      themeConfig: {
        nav: zhNav,
        sidebar: zhSidebar,
        search: { provider: 'local' },
        outline: { level: [2, 3] },
        socialLinks: [{ icon: 'github', link: 'https://github.com/jxpeng98/research-skills' }],
        footer: {
          message: 'Research Skills 文档站',
          copyright: 'MIT License'
        }
      }
    }
  },
  themeConfig: {
    logo: { src: '/mark.svg', alt: 'Research Skills' }
  }
}
