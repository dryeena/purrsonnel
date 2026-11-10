import { Badge, ErrorBox, Form, Modal } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { date } from '../../lib/date';
import { refresh } from '../../lib/events';
import { humanize } from '../../lib/text';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function Notes({ interview, onClose }: { interview: Row; onClose: () => void }) {
  const { data, error } = useData<Row[]>(`/interviews/${interview.id}/notes`);

  return (
    <Modal title={`Notes · ${interview.applicant_name}`} onClose={onClose} wide>
      <p class="modal-description">
        Access follows each note’s visibility. Private notes remain visible only to their author.
      </p>
      {error && <ErrorBox message={error} />}
      <div class="note-list">
        {data?.map((note) => (
          <article key={note.id}>
            <div>
              <strong>{note.author_name}</strong>
              <Badge>{humanize(note.visibility)}</Badge>
            </div>
            <p>{note.body}</p>
            <small>{date(note.created_at, true)}</small>
          </article>
        ))}
      </div>
      <Form
        fields={[
          { name: 'body', label: 'New note', type: 'textarea', required: true, maxLength: 10000 },
          {
            name: 'visibility',
            label: 'Visibility',
            type: 'select',
            required: true,
            options: [
              { value: 'author_only', label: 'Only me' },
              { value: 'interview_panel', label: 'Interview panel' },
              { value: 'hr', label: 'HR with sensitive-data permission' },
            ],
          },
        ]}
        submit="Add note"
        onSubmit={async (values) => {
          await api(`/interviews/${interview.id}/notes`, 'POST', values);
          refresh();
        }}
      />
    </Modal>
  );
}
