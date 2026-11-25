import { fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { Schedules } from '../src/features/schedules';
import { AuthContext } from '../src/lib/auth';

const identity = {
  user: { id: 'volunteer', volunteer_eligible: true },
  grants: [],
  dev_mode: false,
  identities: [],
  csrf_token: 'test',
};
afterEach(() => vi.unstubAllGlobals());
function showSchedule(response: Response) {
  vi.stubGlobal(
    'fetch',
    vi
      .fn()
      .mockImplementation((url: string) =>
        Promise.resolve(
          url.endsWith('/organization/tree')
            ? new Response(JSON.stringify({ departments: [], events: [] }))
            : response,
        ),
      ),
  );
  render(
    <AuthContext.Provider value={identity}>
      <Schedules />
    </AuthContext.Provider>,
  );
}
describe('schedules', () => {
  it('shows an empty state when there are no shifts', async () => {
    showSchedule(new Response('[]'));
    expect(await screen.findByText('No shifts scheduled yet')).toBeVisible();
  });
  it('shows an error when the schedule request fails', async () => {
    showSchedule(new Response('{"detail":"Schedule unavailable"}', { status: 503 }));
    expect(await screen.findByText('Schedule unavailable')).toBeVisible();
  });
  it('shows a volunteer an available opening', async () => {
    showSchedule(
      new Response(
        JSON.stringify([
          {
            id: 'one',
            title: 'Welcome',
            department: 'Registration',
            event: 'FWA',
            starts_at: '2027-05-01T10:00:00Z',
            ends_at: '2027-05-01T12:00:00Z',
            assigned: [],
            assigned_count: 0,
            slots: 1,
            claimed_by_me: false,
          },
        ]),
      ),
    );
    expect(await screen.findByText('Welcome')).toBeVisible();
    expect(screen.getByRole('button', { name: 'Claim opening' })).toBeVisible();
  });
  it('allows a lead to assign staff from another department', async () => {
    const lead = { ...identity, user: { id: 'lead', volunteer_eligible: false } };
    const fetcher = vi.fn().mockImplementation((url: string, options: RequestInit) => {
      if (options.method === 'POST') return Promise.resolve(new Response('{"ok":true}'));
      if (url.endsWith('/organization/tree'))
        return Promise.resolve(new Response('{"departments":[],"events":[]}'));
      if (url.includes('/schedules/staff'))
        return Promise.resolve(new Response('[{"id":"staff","display_name":"River Wolf"}]'));
      return Promise.resolve(
        new Response(
          JSON.stringify([
            {
              id: 'one',
              department_id: 'reg',
              title: 'Welcome',
              department: 'Registration',
              event: 'FWA',
              starts_at: '2027-05-01T10:00:00Z',
              ends_at: '2027-05-01T12:00:00Z',
              assigned: [],
              assigned_count: 0,
              slots: 1,
              can_manage: true,
            },
          ]),
        ),
      );
    });
    vi.stubGlobal('fetch', fetcher);
    render(
      <AuthContext.Provider value={lead}>
        <Schedules />
      </AuthContext.Provider>,
    );
    fireEvent.click(await screen.findByRole('button', { name: 'Assign staff' }));
    const picker = await screen.findByRole('combobox', { name: 'Staff member' });
    await screen.findByRole('option', { name: 'River Wolf' });
    fireEvent.change(picker, { target: { value: 'staff' } });
    fireEvent.submit(screen.getByRole('button', { name: 'Assign' }).closest('form')!);
    await waitFor(() =>
      expect(fetcher).toHaveBeenCalledWith(
        '/api/v1/schedules/one/assignments',
        expect.objectContaining({ method: 'POST', body: '{"user_id":"staff"}' }),
      ),
    );
  });
  it('opens shift creation with the selected calendar day and time', async () => {
    const lead = {
      ...identity,
      grants: [{ permission: 'people.edit', scope_type: 'global' as const, scope_id: null }],
    };
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockImplementation((url: string) =>
          Promise.resolve(
            url.endsWith('/organization/tree')
              ? new Response('{"departments":[{"id":"reg","name":"Registration"}],"events":[]}')
              : new Response('[]'),
          ),
        ),
    );
    render(
      <AuthContext.Provider value={lead}>
        <Schedules />
      </AuthContext.Provider>,
    );
    const slots = await screen.findAllByRole('button', { name: /^Add shift / });
    expect(slots).toHaveLength(168);
    fireEvent.click(slots[0]);
    const starts = screen.getByLabelText(/^Starts/) as HTMLInputElement;
    expect(starts.value).toMatch(/T00:00$/);
  });
});
