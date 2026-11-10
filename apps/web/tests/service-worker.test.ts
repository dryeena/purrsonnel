/** @vitest-environment node */
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import { describe, expect, it, vi } from 'vitest';

function worker() {
  const handlers: Record<string, (event: any) => void> = {};
  const cache = { addAll: vi.fn().mockResolvedValue(undefined) };
  const caches = {
    open: vi.fn().mockResolvedValue(cache),
    match: vi.fn().mockResolvedValue(new Response('offline page')),
    keys: vi.fn().mockResolvedValue([]),
    delete: vi.fn().mockResolvedValue(true),
  };
  const self = {
    location: { origin: 'https://purrsonnel.example' },
    addEventListener: (type: string, handler: (event: any) => void) => {
      handlers[type] = handler;
    },
    registration: { showNotification: vi.fn().mockResolvedValue(undefined) },
    clients: {
      claim: vi.fn().mockResolvedValue(undefined),
      matchAll: vi.fn().mockResolvedValue([]),
      openWindow: vi.fn().mockResolvedValue(undefined),
    },
    skipWaiting: vi.fn(),
  };
  const fetch = vi.fn().mockResolvedValue(new Response('network'));
  runInNewContext(readFileSync(new URL('../public/sw.js', import.meta.url), 'utf8'), {
    self,
    caches,
    fetch,
    URL,
    Response,
    Promise,
  });
  return { handlers, self, caches, cache, fetch };
}
describe('public-only offline behavior', () => {
  it('pre-caches only public fallback assets', async () => {
    const w = worker();
    let done: Promise<any> = Promise.resolve();
    w.handlers.install({
      waitUntil: (p: Promise<any>) => {
        done = p;
      },
    });
    await done;
    expect(w.cache.addAll).toHaveBeenCalledWith([
      '/offline.html',
      '/offline.css',
      '/branding/favicon.svg',
      '/branding/icon-192.png',
    ]);
  });
  it.each([
    '/api/v1/requests',
    '/api/v1/auth/me',
    '/api/v1/volunteer/approval-preview?token=secret',
    '/health/ready',
  ])('does not intercept or cache %s', (path) => {
    const w = worker();
    const respondWith = vi.fn();
    w.handlers.fetch({
      request: { method: 'GET', url: `https://purrsonnel.example${path}`, mode: 'navigate' },
      respondWith,
    });
    expect(respondWith).not.toHaveBeenCalled();
    expect(w.caches.open).not.toHaveBeenCalled();
  });
  it('never intercepts a mutation', () => {
    const w = worker();
    const respondWith = vi.fn();
    w.handlers.fetch({
      request: { method: 'POST', url: 'https://purrsonnel.example/requests', mode: 'navigate' },
      respondWith,
    });
    expect(respondWith).not.toHaveBeenCalled();
  });
  it('serves a public help page when navigation fails, without saving private pages', async () => {
    const w = worker();
    w.fetch.mockRejectedValue(new Error('offline'));
    let response: Promise<Response> = Promise.resolve(new Response());
    w.handlers.fetch({
      request: { method: 'GET', url: 'https://purrsonnel.example/people', mode: 'navigate' },
      respondWith: (p: Promise<Response>) => {
        response = p;
      },
    });
    expect(await (await response).text()).toBe('offline page');
    expect(w.caches.match).toHaveBeenCalledWith('/offline.html');
    expect(w.caches.open).not.toHaveBeenCalled();
  });
});
describe('phone alerts', () => {
  it.each(['request_update', 'request_reminder', 'request_escalated'])(
    'uses a generic message for %s instead of private payload text',
    (kind) => {
      const w = worker();
      w.handlers.push({
        data: { json: () => ({ kind, body: 'Private personnel details' }) },
        waitUntil: vi.fn(),
      });
      const body = w.self.registration.showNotification.mock.calls[0][1].body;
      expect(body).not.toContain('Private');
      expect(body).not.toBe('A new request is waiting in your inbox.');
    },
  );
  it('shows a visible generic alert, requests vibration, and keeps private text off the lock screen', () => {
    const w = worker();
    w.handlers.push({
      data: {
        json: () => ({
          title: 'Secret HR title',
          body: 'Sensitive applicant details',
          url: '/requests?id=11111111-1111-4111-8111-111111111111',
          tag: 'request-11111111-1111-4111-8111-111111111111',
        }),
      },
      waitUntil: vi.fn(),
    });
    expect(w.self.registration.showNotification).toHaveBeenCalledWith(
      'Purrsonnel request',
      expect.objectContaining({
        body: 'A new request is waiting in your inbox.',
        vibrate: [200, 100, 200],
        renotify: false,
        data: { url: '/requests?id=11111111-1111-4111-8111-111111111111' },
      }),
    );
  });
  it.each([
    'https://evil.example/steal',
    'javascript:alert(1)',
    '/api/v1/users',
    '/requests?id=not-an-id',
  ])('rejects notification destination %s', (url) => {
    const w = worker();
    w.handlers.push({ data: { json: () => ({ url }) }, waitUntil: vi.fn() });
    expect(w.self.registration.showNotification.mock.calls[0][1].data.url).toBe('/requests');
  });
  it('still shows an alert for a malformed push payload', () => {
    const w = worker();
    w.handlers.push({
      data: {
        json: () => {
          throw new Error('bad json');
        },
      },
      waitUntil: vi.fn(),
    });
    expect(w.self.registration.showNotification).toHaveBeenCalledOnce();
  });
  it('focuses an existing app and asks it to open the request without discarding forms', async () => {
    const w = worker();
    const client = {
      url: 'https://purrsonnel.example/volunteer',
      postMessage: vi.fn(),
      focus: vi.fn().mockResolvedValue(undefined),
    };
    w.self.clients.matchAll.mockResolvedValue([client]);
    let done: Promise<any> = Promise.resolve();
    w.handlers.notificationclick({
      notification: {
        close: vi.fn(),
        data: { url: '/requests?id=11111111-1111-4111-8111-111111111111' },
      },
      waitUntil: (p: Promise<any>) => {
        done = p;
      },
    });
    await done;
    expect(client.postMessage).toHaveBeenCalledWith({
      type: 'OPEN_REQUEST',
      url: 'https://purrsonnel.example/requests?id=11111111-1111-4111-8111-111111111111',
    });
    expect(client.focus).toHaveBeenCalledOnce();
    expect(w.self.clients.openWindow).not.toHaveBeenCalled();
  });
});
