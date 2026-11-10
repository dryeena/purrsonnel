import { fireEvent, render, screen, waitFor } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { InstallApp } from '../src/components/install';
import { Form, Modal } from '../src/components/ui';
import { clearInstall, vapidBytes } from '../src/lib/pwa';
import { hasUnsavedChanges } from '../src/lib/unsaved';
afterEach(clearInstall);
describe('saving and leaving forms', () => {
  it('keeps forms editable offline while disabling the save action', () => {
    vi.spyOn(navigator, 'onLine', 'get').mockReturnValue(false);
    render(<Form fields={[{ name: 'title', label: 'Title' }]} onSubmit={async () => {}} />);
    fireEvent.input(screen.getByLabelText('Title'), { target: { value: 'Keep this text' } });
    expect(screen.getByLabelText('Title')).toHaveValue('Keep this text');
    expect(screen.getByRole('button', { name: /save changes/i })).toBeDisabled();
    expect(screen.getByText('Reconnect to save.')).toBeVisible();
  });
  it('protects a dirty dialog from accidental dismissal', () => {
    const close = vi.fn();
    const confirm = vi.spyOn(window, 'confirm').mockReturnValue(false);
    render(
      <Modal title="Edit request" onClose={close}>
        <Form fields={[{ name: 'title', label: 'Title' }]} onSubmit={async () => {}} />
      </Modal>,
    );
    fireEvent.input(screen.getByLabelText('Title'), { target: { value: 'Bring badge stock' } });
    expect(hasUnsavedChanges()).toBe(true);
    fireEvent.click(screen.getByLabelText('Close dialog'));
    expect(confirm).toHaveBeenCalled();
    expect(close).not.toHaveBeenCalled();
  });
  it('clears the unsaved warning after a successful save and shows confirmation', async () => {
    render(<Form fields={[{ name: 'title', label: 'Title' }]} onSubmit={async () => {}} />);
    fireEvent.input(screen.getByLabelText('Title'), { target: { value: 'Badge stock' } });
    fireEvent.submit(screen.getByRole('button', { name: /save changes/i }).closest('form')!);
    expect(await screen.findByRole('status')).toHaveTextContent('Saved');
    expect(hasUnsavedChanges()).toBe(false);
  });
  it('keeps typed text and the unsaved warning when saving fails', async () => {
    render(
      <Form
        fields={[{ name: 'title', label: 'Title' }]}
        onSubmit={async () => {
          throw new Error('Not sent');
        }}
      />,
    );
    fireEvent.input(screen.getByLabelText('Title'), { target: { value: 'Badge stock' } });
    fireEvent.submit(screen.getByRole('button', { name: /save changes/i }).closest('form')!);
    expect(await screen.findByRole('alert')).toHaveTextContent('Not sent');
    expect(screen.getByLabelText('Title')).toHaveValue('Badge stock');
    expect(hasUnsavedChanges()).toBe(true);
  });
  it('asks users to explicitly choose a required option', () => {
    render(
      <Form
        fields={[
          {
            name: 'person',
            label: 'Recipient',
            type: 'select',
            required: true,
            options: [{ value: 'maple', label: 'Maple Bunny' }],
          },
        ]}
        onSubmit={async () => {}}
      />,
    );
    expect(screen.getByLabelText(/Recipient/)).toHaveValue('');
    expect(screen.getByText('Choose an option…')).toBeDisabled();
  });
});
describe('home-screen installation', () => {
  it('provides browser-menu instructions if a native prompt is unavailable', () => {
    render(<InstallApp />);
    fireEvent.click(screen.getByRole('button', { name: /add to home screen/i }));
    expect(screen.getByRole('dialog', { name: 'Keep Purrsonnel close' })).toBeVisible();
    expect(screen.getByText('Open your browser menu.')).toBeVisible();
  });
  it('uses the browser install prompt only after an explicit click', async () => {
    const prompt = vi.fn().mockResolvedValue(undefined);
    const event = new Event('beforeinstallprompt', { cancelable: true }) as any;
    event.prompt = prompt;
    event.userChoice = Promise.resolve({ outcome: 'accepted' });
    window.dispatchEvent(event);
    render(<InstallApp />);
    expect(prompt).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole('button', { name: /add to home screen/i }));
    fireEvent.click(screen.getByRole('button', { name: 'Install Purrsonnel' }));
    await waitFor(() => expect(prompt).toHaveBeenCalledOnce());
  });
  it('decodes URL-safe VAPID key bytes for the browser API', () => {
    expect(Array.from(vapidBytes('AAEC_w'))).toEqual([0, 1, 2, 255]);
  });
});
