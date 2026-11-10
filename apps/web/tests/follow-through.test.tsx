import { fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { describe, expect, it, vi } from 'vitest';

import { AlertPreferences } from '../src/components/alert-preferences';
import { DeliverySummary, RequestFollowUp } from '../src/components/request-follow-up';
import { followUpPayload, minuteTime, reminderReady, timeMinute } from '../src/lib/alert-settings';

describe('follow-through policy inputs', () => {
  const now = Date.parse('2026-11-09T12:00:00Z');
  it('converts local clock minutes without applying a browser timezone offset', () => {
    expect(minuteTime(0)).toBe('00:00');
    expect(minuteTime(1439)).toBe('23:59');
    expect(timeMinute('22:15')).toBe(1335);
    expect(timeMinute('08:00')).toBe(480);
    for (const time of ['24:00', '8:00', '12:60', '-1:00'])
      expect(() => timeMinute(time)).toThrow();
  });
  it('sends relative delays for the server clock and rejects an incomplete escalation', () => {
    expect(
      followUpPayload({
        reminder_after: '30',
        escalation_after: '60',
        escalation_user_id: 'person',
      }),
    ).toEqual({
      reminder_after_minutes: 30,
      escalation_after_minutes: 60,
      escalation_user_id: 'person',
    });
    expect(() => followUpPayload({ escalation_after: '60' })).toThrow(/Choose a person/);
    expect(() =>
      followUpPayload({
        reminder_after: '60',
        escalation_after: '30',
        escalation_user_id: 'person',
      }),
    ).toThrow(/after the reminder/);
    expect(
      followUpPayload({ reminder_after: '', escalation_after: '', escalation_user_id: 'unused' }),
    ).toEqual({
      reminder_after_minutes: null,
      escalation_after_minutes: null,
      escalation_user_id: null,
    });
  });
  it('honors the initial wait, last reminder, and terminal states', () => {
    const r = { status: 'open', created_at: '2026-11-09T11:44:00Z' };
    expect(reminderReady(r, now)).toBe(true);
    expect(reminderReady({ ...r, created_at: '2026-11-09T11:46:00Z' }, now)).toBe(false);
    expect(reminderReady({ ...r, last_reminded_at: '2026-11-09T11:50:00Z' }, now)).toBe(false);
    for (const status of ['completed', 'cancelled'])
      expect(reminderReady({ ...r, status }, now)).toBe(false);
  });
});
describe('account phone preferences', () => {
  const defaults = {
    request_alerts: 'all',
    quiet_enabled: false,
    timezone: 'UTC',
    quiet_start: 1320,
    quiet_end: 480,
  };
  it('saves opt-out and quiet hours without invoking browser notification permission', async () => {
    const close = vi.fn();
    const fetcher = vi
      .fn()
      .mockImplementation((_url: string, options: RequestInit) =>
        Promise.resolve(
          new Response(JSON.stringify(options.method === 'GET' ? defaults : { ok: true })),
        ),
      );
    vi.stubGlobal('fetch', fetcher);
    render(<AlertPreferences onClose={close} />);
    await screen.findByRole('combobox', { name: 'Which requests may alert my phone?' });
    fireEvent.change(screen.getByRole('combobox', { name: 'Which requests may alert my phone?' }), {
      target: { value: 'off' },
    });
    fireEvent.click(screen.getByLabelText('Use quiet hours'));
    fireEvent.input(screen.getByRole('textbox', { name: 'Quiet hours timezone' }), {
      target: { value: 'America/Denver' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Save preferences' }));
    await waitFor(() => expect(close).toHaveBeenCalledOnce());
    const call = fetcher.mock.calls.find(([, options]) => options.method === 'PUT');
    expect(JSON.parse(String(call![1].body))).toEqual({
      request_alerts: 'off',
      quiet_enabled: true,
      timezone: 'America/Denver',
      quiet_start: 1320,
      quiet_end: 480,
    });
  });
  it('keeps failed saves visible and leaves the entered choices intact', async () => {
    const close = vi.fn();
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockImplementation((_url: string, options: RequestInit) =>
          Promise.resolve(
            new Response(
              JSON.stringify(
                options.method === 'GET' ? defaults : { detail: 'Choose a recognized timezone' },
              ),
              { status: options.method === 'GET' ? 200 : 400 },
            ),
          ),
        ),
    );
    render(<AlertPreferences onClose={close} />);
    fireEvent.input(await screen.findByRole('textbox', { name: 'Quiet hours timezone' }), {
      target: { value: 'Unknown/Zone' },
    });
    fireEvent.click(screen.getByRole('button', { name: 'Save preferences' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Choose a recognized timezone');
    expect(screen.getByRole('textbox', { name: 'Quiet hours timezone' })).toHaveValue(
      'Unknown/Zone',
    );
    expect(close).not.toHaveBeenCalled();
  });
});
describe('request follow-through presentation', () => {
  it('lets the sender explicitly choose a reminder and escalation recipient before saving', async () => {
    const fetcher = vi.fn().mockImplementation((url: string, options: RequestInit) =>
      Promise.resolve(
        new Response(
          JSON.stringify(
            options.method === 'PUT'
              ? { ok: true }
              : url.endsWith('/recipients')
                ? {
                    people: [
                      { id: 'helper', display_name: 'Aspen Lynx', department: 'Registration' },
                    ],
                  }
                : { follow_up: null, activity: [], delivery: null },
          ),
        ),
      ),
    );
    vi.stubGlobal('fetch', fetcher);
    render(
      <RequestFollowUp
        request={{ id: 'request', status: 'open', created_at: '2020-01-01T00:00:00Z' }}
        isSender
      />,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Schedule follow-up' }));
    await screen.findByLabelText('Send a reminder');
    fireEvent.change(screen.getByLabelText('Send a reminder'), { target: { value: '30' } });
    fireEvent.change(screen.getByLabelText('If still unclaimed, escalate'), {
      target: { value: '60' },
    });
    fireEvent.change(screen.getByLabelText('Escalate to'), { target: { value: 'helper' } });
    fireEvent.click(screen.getByRole('button', { name: 'Save follow-up schedule' }));
    await waitFor(() =>
      expect(fetcher.mock.calls.some(([, options]) => options.method === 'PUT')).toBe(true),
    );
    const call = fetcher.mock.calls.find(([, options]) => options.method === 'PUT');
    expect(JSON.parse(String(call![1].body))).toEqual({
      reminder_after_minutes: 30,
      escalation_after_minutes: 60,
      escalation_user_id: 'helper',
    });
    await waitFor(() => expect(screen.queryByRole('dialog')).not.toBeInTheDocument());
  });
  it('distinguishes push-service acceptance from an actual read or buzz', () => {
    render(
      <DeliverySummary delivery={{ sent: 2, queued: 3, deferred: 1, suppressed: 4, failed: 0 }} />,
    );
    expect(screen.getByText('Accepted by push service')).toBeVisible();
    expect(screen.getByText(/does not confirm that a phone buzzed/)).toBeVisible();
    expect(screen.getByText('Suppressed by preferences')).toBeVisible();
  });
  it('does not offer sender controls to a recipient', async () => {
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockResolvedValue(
          new Response(JSON.stringify({ follow_up: null, activity: [], delivery: null })),
        ),
    );
    render(
      <RequestFollowUp
        request={{ id: 'request', status: 'open', created_at: '2020-01-01T00:00:00Z' }}
        isSender={false}
      />,
    );
    await screen.findByText('No automatic follow-ups scheduled.');
    expect(screen.queryByRole('button', { name: 'Send a reminder' })).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Schedule follow-up' })).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Phone delivery summary')).not.toBeInTheDocument();
  });
  it('records skipped escalation without presenting it as delivery', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        new Response(
          JSON.stringify({
            follow_up: null,
            activity: [
              {
                id: 'event',
                kind: 'escalation_skipped',
                actor_name: null,
                created_at: '2026-11-09T12:00:00Z',
                details: { reason: 'already_accepted_or_closed' },
              },
            ],
            delivery: null,
          }),
        ),
      ),
    );
    render(<RequestFollowUp request={{ id: 'request', status: 'completed' }} isSender />);
    expect(await screen.findByText('Scheduled escalation skipped')).toBeVisible();
    expect(screen.getByText('The request was already accepted or closed.')).toBeVisible();
    expect(screen.queryByRole('button', { name: 'Send a reminder' })).not.toBeInTheDocument();
  });
});
