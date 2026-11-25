import { existsSync } from 'node:fs';
import toolboxReport from '@toolbox-dev/astro-report';
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import routes from './src/data/site-routes.json' with { type: 'json' };

const generated = new URL('./.generated/report.json', import.meta.url);
const reportData = process.env.TOOLBOX_REPORT_DATA ||
  (existsSync(generated) ? generated : new URL('../report-ui/fixtures/t4t.json', import.meta.url));

// Project Pages: https://loganvolkers.github.io/toolbox/
export default defineConfig({
  site: 'https://loganvolkers.github.io',
  base: '/toolbox',
  integrations: [
    toolboxReport({ data: reportData, path: routes.report, aliases: [routes.previousReport], layout: new URL('./src/layouts/ReportLayout.astro', import.meta.url) }),
    starlight({
      title: 'Toolbox',
      description: 'An agent-first internal developer portal, configured in code.',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/loganvolkers/toolbox' },
      ],
      sidebar: [
        { label: 'Start', items: [{ autogenerate: { directory: 'start' } }] },
        { label: 'Operators', items: [{ autogenerate: { directory: 'operators' } }] },
        { label: 'Package authors', items: [{ autogenerate: { directory: 'authors' } }] },
        { label: 'Users', items: [{ autogenerate: { directory: 'users' } }] },
        { label: 'Maintainers', items: [{ autogenerate: { directory: 'maintainers' } }] },
        { label: 'Concepts', items: [{ autogenerate: { directory: 'concepts' } }] },
        { label: 'Agents', items: [{ autogenerate: { directory: 'agents' } }] },
        { label: 'Compare', items: [{ autogenerate: { directory: 'compare' } }] },
        { label: 'Demo', items: [{ autogenerate: { directory: 'demo' } }] },
      ],
    }),
  ],
});
