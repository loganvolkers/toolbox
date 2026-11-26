import { existsSync } from 'node:fs';
import seikikaReport from '@seikika/astro-report';
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import routes from './src/data/site-routes.json' with { type: 'json' };

const generated = new URL('./.generated/report.json', import.meta.url);
const reportData = process.env.SEIKIKA_REPORT_DATA ||
  (existsSync(generated) ? generated : new URL('../report-ui/fixtures/s4s.json', import.meta.url));

export default defineConfig({
  site: 'https://seikika.com',
  integrations: [
    seikikaReport({ data: reportData, path: routes.report, aliases: [routes.previousReport], layout: new URL('./src/layouts/ReportLayout.astro', import.meta.url) }),
    starlight({
      title: 'Seikika',
      description: 'An agent-first internal developer portal, configured in code.',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/seikika/seikika' },
      ],
      sidebar: [
        { label: 'Start', items: [{ autogenerate: { directory: 'start' } }] },
        { label: 'Connectors', items: [{ autogenerate: { directory: 'connectors' } }] },
        { label: 'Packages', items: [{ autogenerate: { directory: 'packages' } }] },
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
