import { act, fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { describe, expect, it } from 'vitest';

import { Feedback } from '../src/components/feedback';
import { Action, Form } from '../src/components/ui';
import { RequestProgress } from '../src/features/requests';
import { isRequestOverdue, relativeTime } from '../src/lib/request-presentation';

describe('request presentation uses persisted state', () => {
  const now = Date.parse('2026-11-09T12:00:00Z');
  it('flags only unfinished work with a due date in the past', () => {
    expect(isRequestOverdue({ status: 'open', due_at: '2026-11-09T11:00:00Z' }, now)).toBe(true);
    expect(isRequestOverdue({ status: 'accepted', due_at: '2026-11-09T11:00:00Z' }, now)).toBe(
      true,
    );
    for (const status of ['completed', 'cancelled'])
      expect(isRequestOverdue({ status, due_at: '2026-11-09T11:00:00Z' }, now)).toBe(false);
    expect(isRequestOverdue({ status: 'open', due_at: '2026-11-09T13:00:00Z' }, now)).toBe(false);
    expect(isRequestOverdue({ status: 'open', due_at: null }, now)).toBe(false);
  });
  it('shows understandable past/future timing without fabricating a date', () => {
    expect(relativeTime('2026-11-09T11:59:55Z', now)).toBe('Just now');
    expect(relativeTime('2026-11-09T11:55:00Z', now)).toBe('5 minutes ago');
    expect(relativeTime('2026-11-09T14:00:00Z', now)).toBe('in 2 hours');
    expect(relativeTime('invalid', now)).toBe('Date unavailable');
  });
  it('distinguishes reached and pending steps without invented timestamps', () => {
    render(<RequestProgress request={{ status: 'accepted' }} />);
    const list = screen.getByRole('list', { name: 'Request progress' });
    expect(list.querySelector('[aria-current="step"]')).toHaveTextContent('Accepted');
    expect(screen.getByText('Not yet')).toBeVisible();
  });
  it('does not display cancellation as completion', () => {
    render(<RequestProgress request={{ status: 'cancelled' }} />);
    expect(screen.getByText(/cancelled.*No further action/)).toBeVisible();
    expect(screen.queryByRole('list', { name: 'Request progress' })).not.toBeInTheDocument();
  });
});
describe('form finishing details', () => {
  it('separates the accessible field name from hints and updates character counts', () => {
    render(
      <Form
        fields={[
          {
            name: 'title',
            label: 'Title',
            required: true,
            maxLength: 10,
            help: 'Keep this short.',
          },
        ]}
        onSubmit={async () => {}}
      />,
    );
    const input = screen.getByRole('textbox', { name: 'Title' });
    expect(input).toHaveAccessibleDescription(/Keep this short/);
    expect(screen.getByText('0 / 10 characters')).toBeVisible();
    fireEvent.input(input, { target: { value: 'Hello' } });
    expect(screen.getByText('5 / 10 characters')).toBeVisible();
    expect(input).toHaveAccessibleName('Title');
  });
  it('moves focus to a failed submission so the error is not missed', async () => {
    render(
      <Form
        fields={[{ name: 'title', label: 'Title' }]}
        onSubmit={async () => {
          throw new Error('This request changed.');
        }}
      />,
    );
    fireEvent.submit(screen.getByRole('button', { name: /save changes/i }).closest('form')!);
    const error = await screen.findByRole('alert');
    await waitFor(() => expect(error.parentElement).toHaveFocus());
  });
});
describe('success feedback', () => {
  it('announces success only after an action completes and can be dismissed', async () => {
    let finish: () => void = () => {};
    render(
      <>
        <Feedback />
        <Action
          successMessage="Request accepted."
          action={() =>
            new Promise<void>((resolve) => {
              finish = resolve;
            })
          }
        >
          Accept
        </Action>
      </>,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Accept' }));
    expect(screen.queryByText('Request accepted.')).not.toBeInTheDocument();
    await act(async () => finish());
    expect(await screen.findByRole('status')).toHaveTextContent('Request accepted.');
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss message' }));
    expect(screen.queryByText('Request accepted.')).not.toBeInTheDocument();
  });
  it('does not announce success when the server rejects an action', async () => {
    render(
      <>
        <Feedback />
        <Action
          successMessage="Accepted"
          action={async () => {
            throw new Error('Another teammate accepted it.');
          }}
        >
          Accept
        </Action>
      </>,
    );
    fireEvent.click(screen.getByRole('button', { name: 'Accept' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('Another teammate');
    expect(screen.queryByRole('status')).not.toBeInTheDocument();
  });
});
