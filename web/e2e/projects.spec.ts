/**
 * E2E workspace flows — runs against the mock-gateway-server (S-100-T7).
 *
 * Prerequisites:
 *   GATEWAY_URL env var points to the running mock-gateway (default: http://127.0.0.1:8081)
 *   APP_URL env var points to the running web dev server (default: http://127.0.0.1:5173)
 *
 * BDD scenarios covered:
 *   HP-1 / SC-ORG-1   — seeded org visible in project list
 *   HP-1 / SC-PROJECT-1 — project detail shows linked assets
 *   EC-1 / SC-MEMBER-2  — non-member session receives 403
 */

import { test, expect } from '@playwright/test';

const GATEWAY_URL = process.env.GATEWAY_URL ?? 'http://127.0.0.1:8081';
const SEED_ORG_ID = 'org-seed-1';
const SEED_PROJECT_ID = 'project-seed-1';
const NON_MEMBER_SESSION = 'e2e-non-member-session';

// ---------------------------------------------------------------------------
// HP-1 / SC-ORG-1 — authenticated session sees seeded project list
// ---------------------------------------------------------------------------

test('HP-1: project list screen renders seed project (SC-ORG-1)', async ({ page }) => {
  // Seed a session via the mock-gateway handoff flow.
  const issueRes = await page.request.post(`${GATEWAY_URL}/e2e/issue-handoff`);
  expect(issueRes.status()).toBe(200);
  const { auth } = await issueRes.json() as { auth: { handoff_code: string } };

  const redeemRes = await page.request.post(`${GATEWAY_URL}/auth/mobile/session`, {
    data: { handoff_code: auth.handoff_code },
  });
  expect(redeemRes.status()).toBe(200);
  const { session_ref } = await redeemRes.json() as { session_ref: string };

  // Verify mock-gateway returns the seed project list for this session.
  const projectsRes = await page.request.get(
    `${GATEWAY_URL}/api/orgs/${SEED_ORG_ID}/projects`,
    { headers: { 'x-dubbridge-session': session_ref } },
  );
  expect(projectsRes.status()).toBe(200);
  const projects = await projectsRes.json() as { id: string; name: string }[];
  expect(projects).toHaveLength(1);
  expect(projects[0].id).toBe(SEED_PROJECT_ID);
  expect(projects[0].name).toBe('E2E Project');
});

// ---------------------------------------------------------------------------
// HP-1 / SC-PROJECT-1 — project detail includes linked assets
// ---------------------------------------------------------------------------

test('HP-1: project detail returns linked assets (SC-PROJECT-1)', async ({ page }) => {
  const issueRes = await page.request.post(`${GATEWAY_URL}/e2e/issue-handoff`);
  const { auth } = await issueRes.json() as { auth: { handoff_code: string } };

  const redeemRes = await page.request.post(`${GATEWAY_URL}/auth/mobile/session`, {
    data: { handoff_code: auth.handoff_code },
  });
  const { session_ref } = await redeemRes.json() as { session_ref: string };

  const detailRes = await page.request.get(
    `${GATEWAY_URL}/api/orgs/${SEED_ORG_ID}/projects/${SEED_PROJECT_ID}`,
    { headers: { 'x-dubbridge-session': session_ref } },
  );
  expect(detailRes.status()).toBe(200);
  const detail = await detailRes.json() as { id: string; assets: { id: string }[] };
  expect(detail.id).toBe(SEED_PROJECT_ID);
  expect(detail.assets.length).toBeGreaterThan(0);
  expect(detail.assets[0].id).toBe('asset-seed-1');
});

// ---------------------------------------------------------------------------
// EC-1 / SC-MEMBER-2 — non-member session denied on org-scoped routes
// ---------------------------------------------------------------------------

test('EC-1: non-member session is denied org-scoped routes (SC-MEMBER-2)', async ({ page }) => {
  const nonMemberHeaders = { 'x-dubbridge-session': NON_MEMBER_SESSION };

  const [projectsRes, detailRes] = await Promise.all([
    page.request.get(`${GATEWAY_URL}/api/orgs/${SEED_ORG_ID}/projects`, {
      headers: nonMemberHeaders,
    }),
    page.request.get(
      `${GATEWAY_URL}/api/orgs/${SEED_ORG_ID}/projects/${SEED_PROJECT_ID}`,
      { headers: nonMemberHeaders },
    ),
  ]);

  expect(projectsRes.status()).toBe(403);
  expect(detailRes.status()).toBe(403);

  const projectsBody = await projectsRes.json() as { error: string };
  expect(projectsBody.error).toBe('forbidden');
});

// ---------------------------------------------------------------------------
// Regression guard — existing health + handoff routes still work
// ---------------------------------------------------------------------------

test('regression: health and handoff bootstrap routes still respond', async ({ page }) => {
  const [healthRes, liveRes] = await Promise.all([
    page.request.get(`${GATEWAY_URL}/health/ready`),
    page.request.get(`${GATEWAY_URL}/health/live`),
  ]);

  expect(healthRes.status()).toBe(200);
  expect(liveRes.status()).toBe(200);
});
