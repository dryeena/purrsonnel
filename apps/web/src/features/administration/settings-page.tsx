import { useState } from 'preact/hooks';

import { Badge, Empty, ErrorBox, Form, Icon, Modal, PageHead, Panel } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { can } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';
import { LevelManager, PermissionManager } from '../configuration';

export function Settings(_: { path?: string }) {
  const me = useAuth();
  const { data: fields, error } = useData<Row[]>(
    can(me, 'settings.manage') ? '/profile-fields' : null,
  );
  const { data: org } = useData<Row>('/organization/tree');
  const { data: events } = useData<Row[]>(can(me, 'settings.manage') ? '/events' : null);
  const [add, setAdd] = useState('');
  if (!can(me, 'settings.manage'))
    return <ErrorBox message="Settings are available to system administrators." />;
  return (
    <>
      <PageHead
        eyebrow="A LITTLE ROOM TO GROW"
        title="Settings"
        description="Configure your organization without changing its code."
      />
      <div class="settings-grid">
        <Panel title="Personnel levels" description="Hierarchy and permissions are independent.">
          <div class="setting-list">
            {org?.levels.map((l: Row) => (
              <div key={l.id}>
                <span>{l.label}</span>
                <Badge>Rank {l.rank}</Badge>
              </div>
            ))}
          </div>
        </Panel>
        <Panel title="Authentication" description="Development provider is active.">
          <div class="setting-content">
            <Badge tone="amber">Development only</Badge>
            <h3>myFWA integration pending</h3>
            <p>
              The adapter will remain unavailable until the official provider configuration and
              claims are confirmed.
            </p>
            <p>Production startup fails closed. No client secrets are displayed here.</p>
          </div>
        </Panel>
        <Panel
          title="Profile field definitions"
          description="Typed fields for your organization."
          action={
            <button type="button" class="text-link" onClick={() => setAdd('field')}>
              + Add field
            </button>
          }
        >
          {error && <ErrorBox message={error} />}
          <div class="setting-list">
            {fields?.length ? (
              fields.map((f) => (
                <div key={f.id}>
                  <span>{f.label}</span>
                  <Badge>{f.type}</Badge>
                </div>
              ))
            ) : (
              <Empty title="Make room for what matters">
                Add a typed profile field as your needs grow.
              </Empty>
            )}
          </div>
        </Panel>
        <Panel
          title="Event editions"
          description="Only active events appear in volunteer time entry and reporting."
          action={
            <button type="button" class="text-link" onClick={() => setAdd('event')}>
              + Add event
            </button>
          }
        >
          <div class="setting-list">
            {events?.length ? (
              events.map((event) => (
                <div key={event.id}>
                  <span>
                    {event.name} <small class="muted">{event.year}</small>
                  </span>
                  <button
                    type="button"
                    class="text-link"
                    onClick={async () => {
                      await api(`/events/${event.id}`, 'PATCH', { active: !event.active });
                      refresh();
                    }}
                  >
                    {event.active ? 'Active' : 'Inactive'}
                  </button>
                </div>
              ))
            ) : (
              <Empty title="No events yet">
                Add an event edition before recording volunteer time.
              </Empty>
            )}
          </div>
        </Panel>
        <Panel
          title="Organization application questions"
          description="New postings snapshot these questions."
        >
          <div class="setting-content">
            <p>Existing application forms stay unchanged when new defaults are added.</p>
            <button type="button" class="button secondary" onClick={() => setAdd('question')}>
              Add default question
              <Icon name="plus" size={16} />
            </button>
          </div>
        </Panel>
      </div>
      <div class="settings-grid settings-extra">
        <LevelManager />
        <PermissionManager />
      </div>
      {add && (
        <Modal
          title={
            add === 'field'
              ? 'Add profile field'
              : add === 'event'
                ? 'Add event edition'
                : 'Add default question'
          }
          onClose={() => setAdd('')}
        >
          <Form
            fields={
              add === 'event'
                ? [
                    { name: 'name', label: 'Event name', required: true },
                    {
                      name: 'year',
                      label: 'Event year',
                      type: 'number',
                      required: true,
                      min: 2000,
                      max: 2100,
                      value: new Date().getFullYear(),
                    },
                    { name: 'starts_at', label: 'Starts', type: 'datetime-local' },
                    { name: 'ends_at', label: 'Ends', type: 'datetime-local' },
                    {
                      name: 'active',
                      label: 'Make this event active now',
                      type: 'checkbox',
                      value: true,
                    },
                  ]
                : [
                    {
                      name: 'key',
                      label: 'Unique identifier (letters, digits, underscore)',
                      required: true,
                    },
                    { name: 'label', label: 'Display label', required: true },
                    {
                      name: 'type',
                      label: 'Type',
                      type: 'select',
                      required: true,
                      options: (add === 'field'
                        ? [
                            'text',
                            'number',
                            'date',
                            'boolean',
                            'single_select',
                            'multi_select',
                            'url',
                          ]
                        : ['text', 'textarea', 'number', 'date', 'boolean', 'url']
                      ).map((v) => ({ value: v, label: v })),
                    },
                    ...(add === 'field'
                      ? [
                          {
                            name: 'visibility',
                            label: 'Visibility',
                            type: 'select',
                            required: true,
                            options: [
                              { value: 'self', label: 'Self and sensitive HR' },
                              { value: 'personnel', label: 'Authorized personnel readers' },
                              { value: 'sensitive', label: 'Self and sensitive HR only' },
                            ],
                          },
                          {
                            name: 'editable_by',
                            label: 'Who may edit?',
                            type: 'select',
                            required: true,
                            options: [
                              { value: 'hr', label: 'HR only' },
                              { value: 'self', label: 'Self or authorized HR' },
                            ],
                          },
                          {
                            name: 'choices_text',
                            label: 'Dropdown choices (one per line)',
                            type: 'textarea',
                            help: 'Used for dropdown and multi-select fields only.',
                            maxLength: 4000,
                          },
                          {
                            name: 'max_length',
                            label: 'Maximum text length',
                            type: 'number',
                            min: 1,
                            max: 10000,
                            value: 4000,
                          },
                        ]
                      : []),
                    ...(add === 'question'
                      ? [
                          { name: 'required', label: 'Required response', type: 'checkbox' },
                          {
                            name: 'sensitive',
                            label: 'Restrict this answer to sensitive-data access',
                            type: 'checkbox',
                          },
                        ]
                      : []),
                  ]
            }
            submit="Create"
            onSubmit={async (b) => {
              if (add === 'field') {
                const choices = String(b.choices_text || '')
                  .split(/\r?\n/)
                  .map((choice) => choice.trim())
                  .filter(Boolean);
                delete b.choices_text;
                if (['single_select', 'multi_select'].includes(b.type) && choices.length === 0) {
                  throw new Error('Add at least one dropdown choice.');
                }
                b.choices = choices;
              }
              if (add === 'event') {
                if (b.starts_at) b.starts_at = new Date(b.starts_at).toISOString();
                if (b.ends_at) b.ends_at = new Date(b.ends_at).toISOString();
              }
              await api(
                add === 'field'
                  ? '/profile-fields'
                  : add === 'event'
                    ? '/events'
                    : '/default-questions',
                'POST',
                b,
              );
              setAdd('');
              refresh();
            }}
          />
        </Modal>
      )}
    </>
  );
}
