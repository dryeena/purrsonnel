import { useState } from 'preact/hooks';

import { Form, Modal, Panel } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { refresh } from '../../lib/events';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

import { QuestionBuilder } from './question-builder';

export function TemplateLibrary() {
  const { data: templates } = useData<Row[]>('/form-templates');
  const { data: org } = useData<Row>('/organization/tree');
  const [editing, setEditing] = useState<Row | null>(null);
  const [questions, setQuestions] = useState<Row[]>([]);
  return (
    <Panel
      title="Application form library"
      description="Reuse a controlled form. New versions preserve existing submissions."
      action={
        <button
          type="button"
          class="text-link"
          onClick={() => {
            setQuestions([]);
            setEditing({});
          }}
        >
          + Create template
        </button>
      }
    >
      <div class="template-library">
        {templates?.map((t) => (
          <div key={t.id}>
            <div>
              <strong>{t.name}</strong>
              <small>
                {t.versions.length} immutable versions · latest v{t.versions[0]?.version_number}
              </small>
            </div>
            <button
              type="button"
              class="button secondary small-button"
              onClick={() => {
                setQuestions(t.versions[0]?.schema_json || []);
                setEditing(t);
              }}
            >
              View / new version
            </button>
          </div>
        ))}
      </div>
      {editing && (
        <Modal
          title={editing.id ? `Form · ${editing.name}` : 'Create application form'}
          onClose={() => setEditing(null)}
          wide
        >
          <p class="modal-description">
            The first saved version is combined with organization defaults. Editing a saved form
            creates a new version; existing postings and applications retain their original form.
          </p>
          <QuestionBuilder questions={questions} setQuestions={setQuestions} />
          <Form
            fields={
              editing.id
                ? []
                : [
                    { name: 'name', label: 'Template name', required: true },
                    {
                      name: 'department_id',
                      label: 'Department (none means organization-wide)',
                      type: 'select',
                      options: org?.departments.map((d: Row) => ({ value: d.id, label: d.name })),
                    },
                  ]
            }
            submit={editing.id ? 'Save new version' : 'Create template'}
            onSubmit={async (b) => {
              if (editing.id)
                await api(`/form-templates/${editing.id}/versions`, 'POST', { schema: questions });
              else
                await api('/form-templates', 'POST', {
                  name: b.name,
                  department_id: b.department_id || null,
                  schema: questions,
                });
              setEditing(null);
              refresh();
            }}
          />
        </Modal>
      )}
    </Panel>
  );
}
