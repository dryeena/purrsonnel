import { fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { describe, expect, it, vi } from 'vitest';

import { Requests } from '../src/features/requests';
import type { Identity } from '../src/lib/api';
import { AuthContext } from '../src/lib/auth';
const identity: Identity = {
  user: { id: 'hr', display_name: 'Juniper Fox' },
  grants: [{ permission: 'requests.send', scope_type: 'global', scope_id: null }],
  dev_mode: true,
  identities: [],
  csrf_token: 'csrf',
};
function mount() {
  render(
    <AuthContext.Provider value={identity}>
      <Requests />
    </AuthContext.Provider>,
  );
}
describe('request inbox', () => {
  it('sends the selected status filter to the API before displaying the next result page', async () => {
    const fetcher = vi
      .fn()
      .mockImplementation((url: string) =>
        Promise.resolve(
          new Response(JSON.stringify(url.endsWith('/push/config') ? { enabled: false } : [])),
        ),
      );
    vi.stubGlobal('fetch', fetcher);
    mount();
    await screen.findByText('A little breathing room.');
    fireEvent.change(screen.getByRole('combobox', { name: 'Filter request status' }), {
      target: { value: 'completed' },
    });
    await waitFor(() =>
      expect(fetcher.mock.calls.some((call) => String(call[0]).includes('status=completed'))).toBe(
        true,
      ),
    );
    expect(await screen.findByText('No requests match these filters')).toBeVisible();
    fireEvent.click(screen.getAllByRole('button', { name: 'Clear filters' })[0]);
    await waitFor(() =>
      expect(screen.getByRole('combobox', { name: 'Filter request status' })).toHaveValue('all'),
    );
  });
  it('explains unconfigured alerts without blocking the inbox', async () => {
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockImplementation((url: string) =>
          Promise.resolve(
            new Response(JSON.stringify(url.endsWith('/push/config') ? { enabled: false } : [])),
          ),
        ),
    );
    mount();
    expect(
      await screen.findByText(/Phone alerts are not available for this workspace yet/),
    ).toBeVisible();
    expect(await screen.findByText('A little breathing room.')).toBeVisible();
    expect(screen.getByRole('button', { name: 'Send a request' })).toBeEnabled();
  });
  it('keeps sending authority separate from receiving requests', async () => {
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockImplementation((url: string) =>
          Promise.resolve(
            new Response(JSON.stringify(url.endsWith('/push/config') ? { enabled: false } : [])),
          ),
        ),
    );
    render(
      <AuthContext.Provider value={{ ...identity, grants: [] }}>
        <Requests />
      </AuthContext.Provider>,
    );
    expect(await screen.findByText('A little breathing room.')).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Send a request' })).not.toBeInTheDocument();
  });
  it('opens a recipient picker with actual server scope and a separate phone-permission choice', async () => {
    const fetcher = vi.fn().mockImplementation((url: string) =>
      Promise.resolve(
        new Response(
          JSON.stringify(
            url.endsWith('/push/config')
              ? { enabled: false }
              : url.endsWith('/recipients')
                ? {
                    departments: [{ id: 'reg', name: 'Registration', recipient_count: 4 }],
                    people: [],
                  }
                : [],
          ),
        ),
      ),
    );
    vi.stubGlobal('fetch', fetcher);
    mount();
    fireEvent.click(screen.getByRole('button', { name: 'Send a request' }));
    expect(
      await screen.findByRole('option', { name: 'Registration · 4 recipients' }),
    ).toBeInTheDocument();
    expect(screen.getByLabelText(/What do you need/)).toHaveValue('');
    await waitFor(() =>
      expect(fetcher).toHaveBeenCalledWith('/api/v1/requests/recipients', expect.anything()),
    );
  });
});
