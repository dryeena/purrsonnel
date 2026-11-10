import { fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { describe, expect, it, vi } from 'vitest';

import { Action, Form, Modal, Status } from '../src/components/ui';
describe('accessible form behavior', () => {
  it('uses the latest dialog close callback after a parent update', () => {
    const original = vi.fn();
    const latest = vi.fn();
    const view = render(
      <Modal title="Current review" onClose={original}>
        Review details
      </Modal>,
    );
    view.rerender(
      <Modal title="Current review" onClose={latest}>
        Updated details
      </Modal>,
    );
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(latest).toHaveBeenCalledOnce();
    expect(original).not.toHaveBeenCalled();
  });
  it('submits multiple choices as an array and allows incomplete draft saves', async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    render(
      <Form
        draft
        fields={[
          {
            name: 'availability',
            label: 'Availability',
            type: 'multi_select',
            options: [
              { value: 'Friday', label: 'Friday' },
              { value: 'Saturday', label: 'Saturday' },
            ],
            value: ['Friday', 'Saturday'],
          },
          { name: 'motivation', label: 'Motivation', required: true },
        ]}
        onSubmit={save}
      />,
    );
    fireEvent.submit(screen.getByRole('button', { name: /save changes/i }).closest('form')!);
    await waitFor(() =>
      expect(save).toHaveBeenCalledWith({ availability: ['Friday', 'Saturday'], motivation: '' }),
    );
  });
  it('preserves booleans and numeric durations instead of submitting strings', async () => {
    const save = vi.fn().mockResolvedValue(undefined);
    render(
      <Form
        fields={[
          { name: 'minutes', label: 'Minutes worked', type: 'number', value: 90 },
          { name: 'eligible', label: 'Volunteer eligible', type: 'checkbox' },
        ]}
        onSubmit={save}
      />,
    );
    fireEvent.click(screen.getByLabelText('Volunteer eligible'));
    fireEvent.submit(screen.getByRole('button', { name: /save changes/i }).closest('form')!);
    await waitFor(() => expect(save).toHaveBeenCalledWith({ minutes: 90, eligible: true }));
  });
  it('displays a server validation error and restores the submit action', async () => {
    const save = vi.fn().mockRejectedValue(new Error('A rejection requires a reason'));
    render(
      <Form
        fields={[{ name: 'note', label: 'Decision note', type: 'textarea' }]}
        onSubmit={save}
        submit="Confirm decision"
      />,
    );
    fireEvent.submit(screen.getByRole('button', { name: /confirm decision/i }).closest('form')!);
    expect(await screen.findByRole('alert')).toHaveTextContent('A rejection requires a reason');
    expect(screen.getByRole('button', { name: /confirm decision/i })).not.toBeDisabled();
  });
  it('prevents a second submission while a request is in flight', async () => {
    let finish: () => void = () => {};
    const save = vi.fn().mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          finish = resolve;
        }),
    );
    render(<Form fields={[]} onSubmit={save} />);
    fireEvent.click(screen.getByRole('button', { name: /save changes/i }));
    await waitFor(() => expect(screen.getByRole('button', { name: /saving/i })).toBeDisabled());
    expect(save).toHaveBeenCalledOnce();
    finish();
  });
  it('renders field hints and semantic required controls', () => {
    render(
      <Form
        fields={[
          {
            name: 'name',
            label: 'Full name',
            required: true,
            help: 'Use the name you want recorded.',
          },
          {
            name: 'consent',
            label: 'I acknowledge this version',
            type: 'checkbox',
            required: true,
          },
        ]}
        onSubmit={async () => {}}
      />,
    );
    expect(screen.getByLabelText(/full name/i)).toBeRequired();
    expect(screen.getByLabelText(/I acknowledge/i)).toBeRequired();
    expect(screen.getByText('Use the name you want recorded.')).toBeVisible();
  });
});
describe('dialogs', () => {
  it('has an accessible name, supports Escape, and restores focus on close', () => {
    const close = vi.fn();
    const trigger = document.createElement('button');
    document.body.append(trigger);
    trigger.focus();
    const { unmount } = render(
      <Modal title="Approve volunteer time" onClose={close}>
        <button type="button">Confirm approval</button>
      </Modal>,
    );
    expect(screen.getByRole('dialog', { name: 'Approve volunteer time' })).toHaveAttribute(
      'aria-modal',
      'true',
    );
    fireEvent.keyDown(document, { key: 'Escape' });
    expect(close).toHaveBeenCalledOnce();
    unmount();
    expect(document.activeElement).toBe(trigger);
    trigger.remove();
  });
  it('keeps keyboard focus inside the dialog', () => {
    render(
      <Modal title="Review" onClose={() => {}}>
        <button type="button">First action</button>
        <button type="button">Last action</button>
      </Modal>,
    );
    const last = screen.getByText('Last action');
    last.focus();
    fireEvent.keyDown(document, { key: 'Tab' });
    expect(screen.getByLabelText('Close dialog')).toHaveFocus();
    fireEvent.keyDown(document, { key: 'Tab', shiftKey: true });
    expect(last).toHaveFocus();
  });
  it('does not close when clicking the document itself', () => {
    const close = vi.fn();
    render(
      <Modal title="Policy" onClose={close}>
        <p>Read the policy</p>
      </Modal>,
    );
    fireEvent.click(screen.getByText('Read the policy'));
    expect(close).not.toHaveBeenCalled();
  });
});
describe('explicit action feedback', () => {
  it('shows a failed mutation instead of implying success', async () => {
    const action = vi.fn().mockRejectedValue(new Error('This entry was already decided'));
    render(<Action action={action}>Approve hours</Action>);
    fireEvent.click(screen.getByRole('button', { name: 'Approve hours' }));
    expect(await screen.findByRole('alert')).toHaveTextContent('already decided');
    expect(screen.getByRole('button', { name: 'Approve hours' })).not.toBeDisabled();
  });
  it('does not rely on color alone to communicate status', () => {
    render(
      <>
        <Status value="approved" />
        <Status value="pending_identity" />
      </>,
    );
    expect(screen.getByText('approved')).toBeVisible();
    expect(screen.getByText('pending identity')).toBeVisible();
  });
});
