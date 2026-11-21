import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// Project Pages: https://loganvolkers.github.io/toolbox/
export default defineConfig({
  site: 'https://loganvolkers.github.io',
  base: '/toolbox',
  integrations: [
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
        { label: 'Demo', items: [{ autogenerate: { directory: 'demo' } }] },
      ],
    }),
  ],
});
