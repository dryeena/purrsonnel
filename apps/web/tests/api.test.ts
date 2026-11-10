import { afterEach, describe, expect, it, vi } from 'vitest';

import type { Identity } from '../src/lib/api';
import { api, authenticate, can, csvCell, downloadCsv, global, setCsrf } from '../src/lib/api';
const identity: Identity = {
  user: { id: 'hr' },
  grants: [
    { permission: 'people.read', scope_type: 'department', scope_id: 'reg' },
    { permission: 'audit.read', scope_type: 'global', scope_id: null },
  ],
  dev_mode: true,
  identities: [],
  csrf_token: 'csrf',
};
afterEach(() => {
  setCsrf('');
  vi.unstubAllGlobals();
});
describe('session transport', () => {
  it('boots once, reads the principal, and sends the returned CSRF token', async () => {
    const fetcher = vi
      .fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({ csrf_token: 'boot-token' })))
      .mockResolvedValueOnce(new Response(JSON.stringify(identity)));
    vi.stubGlobal('fetch', fetcher);
    expect(await authenticate()).toEqual(identity);
    expect(fetcher.mock.calls.map((c) => c[0])).toEqual([
      '/api/v1/auth/dev/bootstrap',
      '/api/v1/auth/me',
    ]);
    expect(fetcher.mock.calls[0][1].credentials).toBe('same-origin');
    expect(fetcher.mock.calls[1][1].headers['X-CSRF-Token']).toBe('boot-token');
  });
  it('sends mutations through the same-origin proxy with no bearer credentials', async () => {
    setCsrf('same-session');
    const fetcher = vi.fn().mockResolvedValue(new Response('{"ok":true}'));
    vi.stubGlobal('fetch', fetcher);
    await api('/volunteer/entries', 'POST', { minutes_worked: 90 });
    const [url, init] = fetcher.mock.calls[0];
    expect(url).toBe('/api/v1/volunteer/entries');
    expect(init.headers['X-CSRF-Token']).toBe('same-session');
    expect(init.headers.Authorization).toBeUndefined();
    expect(JSON.parse(init.body)).toEqual({ minutes_worked: 90 });
    expect(localStorage.length).toBe(0);
  });
  it('surfaces safe API errors without discarding the HTTP status', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(new Response('{"detail":"Code expired"}', { status: 409 })),
    );
    await expect(api('/volunteer/approval-decisions', 'POST', {})).rejects.toMatchObject({
      status: 409,
      message: 'Code expired',
    });
  });
  it('reports a useful error if a proxy returns non-JSON', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('Unavailable', { status: 502 })));
    await expect(api('/dashboard')).rejects.toThrow('Request failed (502)');
  });
});
describe('client permission hints', () => {
  it('shows scoped capabilities without treating them as global', () => {
    expect(can(identity, 'people.read')).toBe(true);
    expect(global(identity, 'people.read')).toBe(false);
    expect(global(identity, 'audit.read')).toBe(true);
    expect(can(identity, 'settings.manage')).toBe(false);
  });
});
describe('spreadsheet export', () => {
  it.each([
    '=HYPERLINK("https://evil.example")',
    '+1+1',
    '-1+1',
    '@SUM(A1:A2)',
    '  =cmd',
    '\tformula',
    '\rformula',
  ])('neutralizes formula prefix %s', (value) => {
    expect(csvCell(value).startsWith('"\'')).toBe(true);
  });
  it('escapes quotes, commas and newlines without losing data', () => {
    expect(csvCell('Maple, "Bunny"\nFWA')).toBe('"Maple, ""Bunny""\nFWA"');
    expect(csvCell(null)).toBe('""');
    expect(csvCell(3)).toBe('"3"');
  });
  it('exports only requested fields and revokes the object URL', () => {
    vi.useFakeTimers();
    const create = vi.fn().mockReturnValue('blob:test');
    const revoke = vi.fn();
    Object.defineProperty(URL, 'createObjectURL', { value: create, configurable: true });
    Object.defineProperty(URL, 'revokeObjectURL', { value: revoke, configurable: true });
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});
    downloadCsv(
      'people.csv',
      [{ display_name: 'Maple', secret: 'never exported' }],
      ['display_name'],
    );
    expect(create).toHaveBeenCalledOnce();
    vi.runAllTimers();
    expect(revoke).toHaveBeenCalledWith('blob:test');
    vi.useRealTimers();
  });
});
