import { useState } from 'preact/hooks';

import type { FieldSpec } from '../../components/ui';
import { Form, Icon, Modal } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { global } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function AddPersonButton() {
  const me = useAuth();
  const [open, setOpen] = useState(false);
  const [post, setPost] = useState('');
  const { data: positions } = useData<Row[]>(
    global(me, 'people.edit') ? '/positions?page_size=100' : null,
  );
  if (!global(me, 'people.edit')) return null;
  const posting = positions?.find((p) => p.id === post);
  const questions: FieldSpec[] =
    posting?.questions.map((q: Row) => ({
      name: q.key,
      label: q.label,
      type: q.type === 'boolean' ? 'checkbox' : q.type === 'single_select' ? 'select' : q.type,
      required: q.required,
      maxLength: q.max_length,
      options: q.options?.map((o: string) => ({ value: o, label: o })),
    })) || [];
  return (
    <>
      <button type="button" class="button primary" onClick={() => setOpen(true)}>
        <Icon name="plus" size={16} />
        Add pending profile
      </button>
      {open && (
        <Modal title="Pre-provision a person" onClose={() => setOpen(false)} wide>
          <p class="modal-description">
            This creates an unlinked profile. It does not create a session or prove ownership of a
            myFWA ID.
          </p>
          <label class="field">
            <span>Optional application intake</span>
            <select value={post} onChange={(e) => setPost(e.currentTarget.value)}>
              <option value="">Profile only</option>
              {positions
                ?.filter((p) => p.status === 'open')
                .map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.title}
                  </option>
                ))}
            </select>
          </label>
          <Form
            key={post}
            fields={[
              { name: 'display_name', label: 'Display name', required: true, maxLength: 120 },
              { name: 'email', label: 'Contact email', type: 'email' },
              ...(post
                ? [
                    {
                      name: 'unverified_myfwa_id',
                      label: 'Supplied myFWA ID (unverified)',
                      maxLength: 120,
                    },
                  ]
                : []),
              ...questions,
            ]}
            submit="Create pending profile"
            onSubmit={async (b) => {
              const answers: Row = {};
              for (const q of questions) answers[q.name] = b[q.name];
              await api('/users', 'POST', {
                display_name: b.display_name,
                email: b.email || null,
                position_posting_id: post || null,
                unverified_myfwa_id: b.unverified_myfwa_id || null,
                answers_json: post ? answers : null,
              });
              setOpen(false);
              refresh();
            }}
          />
        </Modal>
      )}
    </>
  );
}
