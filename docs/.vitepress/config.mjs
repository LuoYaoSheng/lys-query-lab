import { defineConfig } from 'vitepress';

export default defineConfig({
  title: 'QueryLab',
  description: '本地优先数据库客户端',
  lang: 'zh-CN',
  cleanUrls: true,
  ignoreDeadLinks: true,
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
    ],
  },
});
