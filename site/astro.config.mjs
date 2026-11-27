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
      components: { Search: './src/components/SiteSearch.astro' },
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/seikika/seikika' },
      ],
      sidebar: [
        {
          label: 'Guides',
          items: [
            { label: 'Start', collapsed: true, items: [
              'start/quickstart', 'start/install', 'start/run', 'start/portal',
              'start/versions', 'start/troubleshooting',
            ] },
            { label: 'Operators', collapsed: true, items: [
              'operators', 'operators/portal', 'operators/ci', 'operators/deploy', 'operators/access',
              'operators/secrets', 'operators/actions', 'operators/federate-actions',
              'operators/publish', 'operators/standards', 'operators/assessments',
            ] },
            { label: 'Package authors', collapsed: true, items: [
              'authors', 'authors/connectors', 'authors/secrets',
            ] },
            { label: 'Users', collapsed: true, items: ['users', 'users/actions', 'users/refresh'] },
            { label: 'Agents', collapsed: true, items: [{ autogenerate: { directory: 'agents' } }] },
            { label: 'Maintainers', collapsed: true, items: [{ autogenerate: { directory: 'maintainers' } }] },
          ],
        },
        {
          label: 'Reference',
          items: [
            { slug: 'reference', label: 'Reference overview' },
            'users/api', 'reference/mcp', 'start/cli', 'reference/sdks',
            'authors/npm', 'operators/manifest', 'reference/contracts',
          ],
        },
        { label: 'Connectors', items: [{ autogenerate: { directory: 'connectors' } }] },
        { label: 'Packages', items: [{ autogenerate: { directory: 'packages' } }] },
        { label: 'Concepts', items: [{ autogenerate: { directory: 'concepts' } }] },
        { label: 'Compare', items: [{ autogenerate: { directory: 'compare' } }] },
        { label: 'Demo', items: [{ autogenerate: { directory: 'demo' } }] },
      ],
    }),
  ],
});
