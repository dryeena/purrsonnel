import { render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { Dashboard } from '../src/features/dashboard';
import type { Identity } from '../src/lib/api';
import { AuthContext } from '../src/lib/auth';
const identity: Identity = {
  user: {
    id: 'hr',
    display_name: 'Juniper Fox',
    departments: [],
    service_years: [2021, 2023],
    verified_years: 2,
  },
  grants: [{ permission: 'people.read', scope_type: 'global', scope_id: null }],
  csrf_token: 'x',
  identities: [],
  dev_mode: true,
};
function fetchData() {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockImplementation((url: string) =>
      Promise.resolve(
        new Response(
          JSON.stringify(
            url.endsWith('/dashboard')
              ? {
                  counts: {
                    people: 10,
                    staff: 5,
                    pending_approvals: 1,
                    applications: 1,
                    pending_terms: 1,
                    volunteer_minutes: 90,
                  },
                  departments: [{ name: 'Registration', count: 4 }],
                  notifications: [],
                }
              : url.includes('/users')
                ? {
                    items: [
                      {
                        id: 'maple',
                        display_name: 'Maple Bunny',
                        myfwa_id: 'DEV-FWA-0007',
                        department: 'Registration',
                        status: 'active',
                      },
                    ],
                  }
                : url.includes('/terms/')
                  ? [
                      {
                        version_id: 'v1',
                        title: 'Community code of conduct',
                        version_number: 1,
                        signed_at: null,
                      },
                    ]
                  : [],
          ),
        ),
      ),
    ),
  );
}
afterEach(() => vi.unstubAllGlobals());
describe('dashboard priorities', () => {
  it('shows real API counts, clear next steps and the supplied branding', async () => {
    fetchData();
    render(
      <AuthContext.Provider value={identity}>
        <Dashboard />
      </AuthContext.Provider>,
    );
    expect(await screen.findByText('Hello, Juniper.')).toBeVisible();
    expect(screen.getByText('10')).toBeVisible();
    expect(screen.getByText('1 volunteer entry needs a review')).toBeVisible();
    expect(screen.getByText('Community code of conduct')).toBeVisible();
    expect(screen.getByAltText('Purrsonnel cat peeking over a personnel ID badge')).toHaveAttribute(
      'src',
      '/branding/purrsonnel-logo.png',
    );
    expect(screen.getByText('Maple Bunny')).toBeVisible();
  });
  it('shows a member their actual verified service count and own hours', async () => {
    fetchData();
    const member = { ...identity, grants: [] };
    render(
      <AuthContext.Provider value={member}>
        <Dashboard />
      </AuthContext.Provider>,
    );
    expect(await screen.findByText('Hello, Juniper.')).toBeVisible();
    expect(screen.getByText('Verified service years')).toBeVisible();
    expect(screen.getByText('2')).toBeVisible();
    expect(screen.getByText('1.5')).toBeVisible();
    expect(screen.queryByText('People in your scope')).not.toBeInTheDocument();
  });
  it('shows a useful error instead of zeroing failed metrics', async () => {
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockResolvedValue(new Response('{"detail":"Database is unavailable"}', { status: 503 })),
    );
    render(
      <AuthContext.Provider value={identity}>
        <Dashboard />
      </AuthContext.Provider>,
    );
    expect(await screen.findByRole('alert')).toHaveTextContent('Database is unavailable');
    expect(screen.queryByText('People in your scope')).not.toBeInTheDocument();
  });
});
