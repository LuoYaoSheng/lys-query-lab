import { defineConfig } from 'vitepress';

export default defineConfig({
  title: 'QueryLab',
  description: '本地优先数据库客户端',
  lang: 'zh-CN',
  base: '/',
  cleanUrls: true,
  ignoreDeadLinks: true,

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' }],
    ['meta', { name: 'author', content: 'LuoYaoSheng' }],
    ['meta', { name: 'keywords', content: '数据库客户端,MySQL,MariaDB,SQL,桌面工具,Tauri,QueryLab' }],
    ['meta', { property: 'og:type',        content: 'website' }],
    ['meta', { property: 'og:site_name',   content: 'QueryLab' }],
    ['meta', { property: 'og:title',       content: 'QueryLab — 本地优先数据库客户端' }],
    ['meta', { property: 'og:description', content: '聚焦 MySQL / MariaDB 的本地优先数据库客户端，轻量安全。' }],
    ['meta', { property: 'og:url',         content: 'https://query.open.i2kai.com/' }],
    ['meta', { property: 'og:locale',      content: 'zh_CN' }],
    ['meta', { name: 'twitter:card',        content: 'summary_large_image' }],
    ['meta', { name: 'twitter:title',       content: 'QueryLab — 本地优先数据库客户端' }],
    ['meta', { name: 'twitter:description', content: '聚焦 MySQL / MariaDB 的本地优先数据库客户端。' }],
    ['meta', { name: 'theme-color', content: '#646cff' }],
  ],

  themeConfig: {
    nav: [
      { text: '首页', link: '/' },
      { text: 'PRD', link: '/PRD' },
      { text: 'SQL 补全策略', link: '/API_SQL_补全策略' },
    ],
    sidebar: [
      {
        text: '文档',
        items: [
          { text: '简介', link: '/README' },
          { text: 'PRD', link: '/PRD' },
          { text: 'SQL 补全策略', link: '/API_SQL_补全策略' },
        ],
      },
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/LuoYaoSheng/lys-query-lab' },
      { icon: 'github', link: 'https://gitee.com/luoyaosheng/lys-query-lab', ariaLabel: 'Gitee' },
    ],
  },
});
