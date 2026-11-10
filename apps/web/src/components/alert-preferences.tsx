import { useData } from '../hooks/use-data';
import { minuteTime, timeMinute } from '../lib/alert-settings';
import { notify } from '../lib/feedback';
import { api } from '../services/api-client';
import type { Row } from '../types/api';

import { ErrorBox, Form, Loading, Modal } from './ui';
export function AlertPreferences({ onClose }: { onClose: () => void }) {
  const { data, error, reload } = useData<Row>('/notification-preferences');
  return (
    <Modal title="Phone alert preferences" onClose={onClose}>
      <p class="modal-description">
        These settings apply to all your subscribed devices. Every request and update stays in your
        Purrsonnel inbox.
      </p>
      {error ? (
        <>
          <ErrorBox message={error} />
          <button type="button" class="button secondary" onClick={reload}>
            Try again
          </button>
        </>
      ) : !data ? (
        <Loading />
      ) : (
        <Form
          fields={[
            {
              name: 'request_alerts',
              label: 'Which requests may alert my phone?',
              type: 'select',
              required: true,
              value: data.request_alerts,
              options: [
                { value: 'all', label: 'All requests and their updates' },
                { value: 'urgent', label: 'Urgent requests and their updates only' },
                { value: 'off', label: 'Inbox only · no phone alerts' },
              ],
            },
            {
              name: 'quiet_enabled',
              label: 'Use quiet hours',
              type: 'checkbox',
              value: data.quiet_enabled,
              help: 'Phone alerts wait until quiet hours end, including urgent requests. Old or closed requests may expire instead.',
            },
            {
              name: 'quiet_start',
              label: 'Quiet hours start',
              type: 'time',
              required: true,
              value: minuteTime(data.quiet_start),
            },
            {
              name: 'quiet_end',
              label: 'Quiet hours end',
              type: 'time',
              required: true,
              value: minuteTime(data.quiet_end),
            },
            {
              name: 'timezone',
              label: 'Quiet hours timezone',
              required: true,
              maxLength: 100,
              value: data.updated_at
                ? data.timezone
                : Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC',
              help: 'Use a timezone name such as America/Denver. This stays fixed when you travel; update it here when needed.',
            },
          ]}
          submit="Save preferences"
          onSubmit={async (values) => {
            const quiet_start = timeMinute(values.quiet_start),
              quiet_end = timeMinute(values.quiet_end);
            if (quiet_start === quiet_end)
              throw new Error('Choose different quiet-hour start and end times.');
            await api('/notification-preferences', 'PUT', {
              request_alerts: values.request_alerts,
              quiet_enabled: values.quiet_enabled,
              timezone: values.timezone.trim(),
              quiet_start,
              quiet_end,
            });
            notify('Phone alert preferences saved. Your inbox stays available.');
            onClose();
          }}
        />
      )}
      <p class="small-note">
        Your phone’s sound, vibration, and Focus settings still apply. Enabling alerts later does
        not resend previously suppressed notifications.
      </p>
    </Modal>
  );
}
