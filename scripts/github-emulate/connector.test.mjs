import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { after, before, beforeEach, test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { createEmulator } from 'emulate';

const binary = fileURLToPath(new URL('../../target/debug/seikika-connector-github', import.meta.url));
const org = 'seikika-fixture';
const token = 'emulate-test-token';
const appId = 12345;
const installationId = 67890;
const activeRepo = `/repos/${org}/active`;
const archivedRepo = `/repos/${org}/archived`;
const unknownId = 2147483647;
const preferredOwners = {
  repo: 'active',
  rules: [
    { pattern: '*', owners: ['@seikika-fixture/core'] },
    { pattern: '/docs/', owners: ['@seikika-fixture/docs', '@direct-admin'] },
  ],
};
let github;
let activeId;
let archivedId;
let preferredSha;
let rootSha;

async function api(path, method = 'GET', body) {
  const response = await fetch(`${github.url}${path}`, {
    method,
    headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
    redirect: 'error',
    signal: AbortSignal.timeout(5_000),
  });
  const text = await response.text();
  assert.ok(response.ok, `${method} ${path}: HTTP ${response.status} ${text}`);
  return text ? JSON.parse(text) : undefined;
}

async function writeCodeowners(path, text) {
  const { content } = await api(`${activeRepo}/contents/${path}`, 'PUT', {
    message: `Add ${path}`,
    content: Buffer.from(text).toString('base64'),
  });
  return content.sha;
}

before(async () => {
  // Keep the advertised URL on this listener even when the shell is configured for portless.
  delete process.env.EMULATE_BASE_URL;
  delete process.env.PORTLESS_URL;
  github = await createEmulator({
    service: 'github',
    hostname: '127.0.0.1',
    port: 0,
    seed: {
      tokens: { [token]: { login: 'maintainer', scopes: ['repo', 'admin:org'] } },
      github: {
        strict_scopes: true,
        users: [{ login: 'maintainer' }, { login: 'direct-admin' }],
        orgs: [{ login: org, members: [{ login: 'maintainer', role: 'admin' }] }],
        repos: [
          { owner: org, name: 'active', private: true, default_branch: 'trunk' },
          { owner: org, name: 'archived', private: true },
        ],
        apps: [{
          app_id: appId,
          slug: 'seikika-fixture-app',
          name: 'Seikika Fixture App',
          permissions: { contents: 'read' },
          installations: [{
            installation_id: installationId,
            account: org,
            repository_selection: 'selected',
            repositories: ['active'],
          }],
        }],
      },
    },
  });
});

beforeEach(async () => {
  github.reset();
  ({ id: activeId } = await api(activeRepo));
  ({ id: archivedId } = await api(archivedRepo, 'PATCH', { archived: true }));
  await api(`${activeRepo}/collaborators/direct-admin`, 'PUT', { permission: 'admin' });
  preferredSha = await writeCodeowners('.github/CODEOWNERS',
    '# preferred location\n* @seikika-fixture/core\n/docs/ @seikika-fixture/docs @direct-admin # inline comment\n');
  rootSha = await writeCodeowners('CODEOWNERS', '* @seikika-fixture/root\n');
  await writeCodeowners('docs/CODEOWNERS', '* @seikika-fixture/fallback\n');
});

after(async () => {
  await github?.close();
});

async function refresh(kind, externalIds, credentials = { token }) {
  const command = { type: 'command', command: 'refresh', command_id: 'refresh', kind, external_ids: externalIds };
  // The connector receives only the synthetic config, even if the developer has GITHUB_TOKEN set.
  const env = { SEIKIKA_CONNECTOR_CONFIG: JSON.stringify({ org, apiUrl: github.url, ...credentials }) };
  const stdout = await new Promise((resolve, reject) => {
    const child = execFile(binary, { env, timeout: 15_000 }, (error, output, stderr) => {
      if (error) {
        const message = error.code === 'ENOENT'
          ? 'Build the connector with cargo build --locked -p seikika-connector-github.'
          : `Running the connector failed. ${stderr}`;
        reject(new Error(message, { cause: error }));
      } else {
        resolve(output);
      }
    });
    // execFile reports process failures separately, including an early exit before stdin is read.
    child.stdin.on('error', reject);
    child.stdin.end(`${JSON.stringify(command)}\n`);
  });
  const [registration, ...messages] = stdout.trim().split('\n').map((line) => JSON.parse(line));
  assert.equal(registration.type, 'register');
  assert.equal(registration.connector, 'github');
  assert.ok(registration.capabilities.refresh[kind].includes('external_ids'));
  return messages;
}

function assertRefresh(messages, kind, records, missing = []) {
  const command_id = 'refresh';
  assert.deepEqual(messages, [
    { type: 'ack', command_id },
    ...records.map(([externalId, data]) => ({ type: 'record', command_id, kind, externalId, data })),
    ...missing.map((externalId) => ({ type: 'not_found', command_id, kind, externalId })),
    { type: 'complete', command_id, status: 'succeeded', counts: { records: records.length, not_found: missing.length } },
  ]);
}

function assertFailedRefresh(messages, error) {
  assert.deepEqual(messages, [
    { type: 'ack', command_id: 'refresh' },
    {
      type: 'complete', command_id: 'refresh', status: 'failed', counts: { records: 0, not_found: 0 },
      errors: [{ code: 'source', message: error }],
    },
  ]);
}

test('repository refresh reads branch protection, admins and archive status by ID', async () => {
  await api(`${activeRepo}/branches/trunk/protection`, 'PUT', { enforce_admins: true });
  const active = `github:repo/${activeId}`;
  const archived = `github:repo/${archivedId}`;
  const unknown = `github:repo/${unknownId}`;
  const messages = await refresh('repository', [active, archived, unknown]);
  assertRefresh(messages, 'repository', [
    [active, { id: activeId, name: 'active', archived: false, default_branch_protected: true, admins: ['direct-admin'] }],
    [archived, { id: archivedId, name: 'archived', archived: true, default_branch_protected: false, admins: [] }],
  ], [unknown]);
});

test('CODEOWNERS refresh follows precedence as files are removed', async () => {
  const active = `github:codeowners/${activeId}`;
  const missing = `github:codeowners/${archivedId}`;
  assertRefresh(await refresh('codeowners', [active, missing]), 'codeowners', [[active, preferredOwners]], [missing]);

  await api(`${activeRepo}/contents/.github/CODEOWNERS`, 'DELETE', {
    message: 'Remove preferred CODEOWNERS', sha: preferredSha,
  });
  assertRefresh(await refresh('codeowners', [active]), 'codeowners', [[active, {
    repo: 'active', rules: [{ pattern: '*', owners: ['@seikika-fixture/root'] }],
  }]]);

  await api(`${activeRepo}/contents/CODEOWNERS`, 'DELETE', {
    message: 'Remove root CODEOWNERS', sha: rootSha,
  });
  assertRefresh(await refresh('codeowners', [active]), 'codeowners', [[active, {
    repo: 'active', rules: [{ pattern: '*', owners: ['@seikika-fixture/fallback'] }],
  }]]);
});

test('GitHub App exchange grants access to the selected private repository', async () => {
  const appKey = github.generatedSecrets.find(({ kind, id }) => (
    kind === 'github.app_private_key' && id === String(appId)
  ));
  assert.ok(appKey, 'Emulate generated a key for the fixture App');
  const credentials = { appId: String(appId), privateKey: appKey.value, installationId: String(installationId) };
  const kind = 'codeowners';
  const active = `github:codeowners/${activeId}`;
  assertRefresh(await refresh(kind, [active], credentials), kind, [[active, preferredOwners]]);

  const { installation_tokens: issued } = await api('/_emulate/installation-tokens');
  assert.ok(issued.length > 0, 'the connector exchanged an App JWT for an installation token');
  for (const entry of issued) {
    assert.equal(entry.installation.id, installationId);
    assert.deepEqual(entry.repository_ids, [activeId]);
  }
  assertFailedRefresh(await refresh(kind, [`github:codeowners/${archivedId}`], credentials),
    `GET /repositories/${archivedId}: HTTP 403`);
});

test('an invalid token fails the refresh with GitHub HTTP status', async () => {
  const messages = await refresh('repository', [`github:repo/${activeId}`], { token: 'unknown-test-token' });
  assertFailedRefresh(messages, `GET /repositories/${activeId}: HTTP 401`);
});
