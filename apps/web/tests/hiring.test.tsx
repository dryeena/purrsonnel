import { fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { describe, expect, it, vi } from 'vitest';

import { Finalize } from '../src/features/hiring/finalize-dialog';

describe('final hiring review', () => {
  it('renders panel recommendations and submits only the explicitly confirmed decision', async () => {
    const fetcher = vi.fn().mockImplementation((_url: string, options: RequestInit) =>
      Promise.resolve(
        new Response(
          JSON.stringify(
            options.method === 'GET'
              ? [
                  {
                    id: 'interview',
                    application_id: 'application',
                    scheduled_start_at: '2026-11-09T12:00:00Z',
                    timezone: 'UTC',
                    panel: [
                      { user_id: 'reviewer', display_name: 'Aspen Lynx', response: 'approve' },
                    ],
                  },
                ]
              : { status: 'approved' },
          ),
        ),
      ),
    );
    vi.stubGlobal('fetch', fetcher);
    render(
      <Finalize
        application={{ id: 'application', applicant_name: 'Maple Bunny', position: 'Registration' }}
        onClose={vi.fn()}
      />,
    );
    expect(await screen.findByText('Aspen Lynx')).toBeVisible();
    expect(fetcher.mock.calls.some(([, options]) => options.method === 'POST')).toBe(false);

    fireEvent.change(screen.getByRole('combobox', { name: 'Final decision' }), {
      target: { value: 'approve' },
    });
    fireEvent.input(screen.getByRole('textbox', { name: 'Decision reason / HR override reason' }), {
      target: { value: 'Panel reviewed; ready to help.' },
    });
    fireEvent.click(screen.getByRole('checkbox'));
    fireEvent.submit(
      screen.getByRole('button', { name: 'Confirm final decision' }).closest('form')!,
    );

    await waitFor(() => expect(screen.getByText(/Application approved/)).toBeVisible());
    const submission = fetcher.mock.calls.find(([, options]) => options.method === 'POST');
    expect(submission?.[0]).toBe('/api/v1/applications/application/finalize');
    expect(JSON.parse(String(submission?.[1].body))).toMatchObject({
      decision: 'approve',
      reason: 'Panel reviewed; ready to help.',
    });
  });
});
