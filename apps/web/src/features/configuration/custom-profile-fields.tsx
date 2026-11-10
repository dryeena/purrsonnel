import { useState } from 'preact/hooks';

import type { FieldSpec } from '../../components/ui';
import { Form } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { global } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function CustomProfileFields({ personId }: { personId: string }) {
  const me = useAuth();
  const { data } = useData<Row[]>(`/users/${personId}/profile-fields`);
  const [open, setOpen] = useState(false);
  if (!data?.length) return null;
  const editable = data.filter(
    (f) => global(me, 'people.edit') || (personId === me.user.id && f.editable_by === 'self'),
  );
  const fields: FieldSpec[] = editable.map((f) => ({
    name: f.id,
    label: f.label,
    type: f.type === 'boolean' ? 'checkbox' : f.type === 'single_select' ? 'select' : f.type,
    value: f.value,
    maxLength: f.max_length || 4000,
    options: f.choices?.map((v: string) => ({ value: v, label: v })),
  }));
  return (
    <div class="detail-section">
      <div class="detail-section-heading">
        <h3>Organization profile fields</h3>
        {editable.length > 0 && (
          <button
            type="button"
            class="button secondary small-button"
            onClick={() => setOpen(!open)}
          >
            {open ? 'Close editor' : 'Edit profile fields'}
          </button>
        )}
      </div>
      <div class="detail-grid">
        {data.map((f) => (
          <div key={f.id}>
            <span>{f.label}</span>
            <strong>{f.value === null ? 'Not provided' : String(f.value)}</strong>
          </div>
        ))}
      </div>
      {open && (
        <Form
          fields={fields}
          onSubmit={async (b) => {
            await api(`/users/${personId}/profile-fields`, 'PATCH', { values: b });
            setOpen(false);
            refresh();
          }}
        />
      )}
    </div>
  );
}
