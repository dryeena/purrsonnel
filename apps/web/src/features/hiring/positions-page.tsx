import { useState } from 'preact/hooks';

import {
  Action,
  Badge,
  Empty,
  ErrorBox,
  Form,
  Icon,
  Loading,
  Modal,
  PageHead,
  Panel,
  Status,
} from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { can } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';
import { PositionOwner } from '../lifecycle';

import { Apply } from './apply-dialog';
import { QuestionBuilder } from './question-builder';
import { TemplateLibrary } from './template-library';

export function Positions(_: { path?: string }) {
  const me = useAuth();
  const { data, error, loading } = useData<Row[]>('/positions?page_size=100');
  const { data: org } = useData<Row>('/organization/tree');
  const { data: people } = useData<Row>(
    can(me, 'positions.manage') ? '/users?page_size=100' : null,
  );
  const [create, setCreate] = useState(false);
  const [apply, setApply] = useState<Row | null>(null);
  const [questions, setQuestions] = useState<Row[]>([]);
  const { data: templates } = useData<Row[]>(
    can(me, 'positions.manage') ? '/form-templates' : null,
  );
  return (
    <>
      <PageHead
        eyebrow="FIND YOUR NEXT CHAPTER"
        title="Positions"
        description="Meaningful work. Memorable weekends. A team that’s glad you’re here."
      >
        {can(me, 'positions.manage') && (
          <button type="button" class="button primary" onClick={() => setCreate(true)}>
            <Icon name="plus" size={16} />
            Create position
          </button>
        )}
      </PageHead>
      {error ? (
        <ErrorBox message={error} />
      ) : loading ? (
        <Loading />
      ) : data?.length ? (
        <div class="card-grid">
          {data.map((p) => (
            <Panel key={p.id}>
              <div class="position-card">
                <div class="policy-top">
                  <Badge tone="plum">{p.department}</Badge>
                  <Status value={p.status} />
                </div>
                <span class="position-level">
                  {p.level}
                  {p.is_leadership ? ' · Leadership' : ''}
                </span>
                <h2>{p.title}</h2>
                <p class="document-excerpt">{p.description_markdown}</p>
                <div class="position-meta">
                  <span>
                    <Icon name="people" size={16} />
                    {p.owner_name}
                  </span>
                  {p.application_count !== null && <span>{p.application_count} applications</span>}
                </div>
                <div class="row-actions">
                  <PositionOwner position={p} />
                  {p.status === 'open' && (
                    <button type="button" class="button primary" onClick={() => setApply(p)}>
                      Start application
                      <Icon name="arrow" size={16} />
                    </button>
                  )}
                  {can(me, 'positions.manage') && p.status !== 'archived' && (
                    <Action
                      action={async () => {
                        await api(`/positions/${p.id}`, 'PATCH', {
                          status: p.status === 'open' ? 'closed' : 'open',
                        });
                        refresh();
                      }}
                    >
                      {p.status === 'open' ? 'Close posting' : 'Open posting'}
                    </Action>
                  )}
                </div>
              </div>
            </Panel>
          ))}
        </div>
      ) : (
        <Panel>
          <Empty title="A new opportunity is on its way">Open positions will appear here.</Empty>
        </Panel>
      )}
      {can(me, 'positions.manage') && <TemplateLibrary />}
      {create && (
        <Modal title="Create a position" onClose={() => setCreate(false)} wide>
          <p class="modal-description">
            Organization default questions are included automatically. Add role-specific questions
            below.
          </p>
          <QuestionBuilder questions={questions} setQuestions={setQuestions} />
          <Form
            fields={[
              { name: 'title', label: 'Position title', required: true, maxLength: 160 },
              {
                name: 'form_template_version_id',
                label: 'Use a saved form (optional)',
                type: 'select',
                options: templates?.map((t) => ({ value: t.versions[0]?.id, label: t.name })),
              },
              {
                name: 'department_id',
                label: 'Department',
                type: 'select',
                required: true,
                options: org?.departments.map((d: Row) => ({ value: d.id, label: d.name })),
              },
              {
                name: 'position_level_id',
                label: 'Position level',
                type: 'select',
                required: true,
                options: org?.position_levels.map((d: Row) => ({ value: d.id, label: d.label })),
              },
              { name: 'is_leadership', label: 'This is a leadership position', type: 'checkbox' },
              {
                name: 'hiring_owner_user_id',
                label: 'Hiring owner',
                type: 'select',
                required: true,
                value: me.user.id,
                options: people?.items.map((u: Row) => ({ value: u.id, label: u.display_name })),
              },
              {
                name: 'description_markdown',
                label: 'About this role',
                type: 'textarea',
                required: true,
                maxLength: 20000,
              },
              {
                name: 'status',
                label: 'Initial status',
                type: 'select',
                required: true,
                options: [
                  { value: 'draft', label: 'Save as draft' },
                  { value: 'open', label: 'Open for applications' },
                ],
              },
            ]}
            submit="Create position"
            onSubmit={async (b) => {
              await api('/positions', 'POST', {
                ...b,
                form_template_version_id: b.form_template_version_id || null,
                questions,
              });
              setCreate(false);
              setQuestions([]);
              refresh();
            }}
          />
        </Modal>
      )}
      {apply && <Apply position={apply} onClose={() => setApply(null)} />}
    </>
  );
}
