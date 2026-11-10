import { useState } from 'preact/hooks';

import {
  Avatar,
  Badge,
  ErrorBox,
  Form,
  Icon,
  Loading,
  Modal,
  PageHead,
  Panel,
} from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { can } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';
import { DepartmentEditor } from '../lifecycle';

export function Organization(_: { path?: string }) {
  const me = useAuth();
  const { data, error, loading } = useData<Row>('/organization/tree');
  const [create, setCreate] = useState('');
  const [department, setDepartment] = useState<Row | null>(null);
  const { data: members } = useData<Row>(
    department ? `/users?department=${department.id}&page_size=100` : null,
  );
  return (
    <>
      <PageHead
        eyebrow="BETTER TOGETHER"
        title="Organization"
        description="A place for every team, and a clear view of how we fit together."
      >
        {can(me, 'organization.manage') && (
          <>
            <button type="button" class="button secondary" onClick={() => setCreate('division')}>
              Add division
            </button>
            <button type="button" class="button primary" onClick={() => setCreate('department')}>
              <Icon name="plus" size={16} />
              Add department
            </button>
          </>
        )}
      </PageHead>
      {error ? (
        <ErrorBox message={error} />
      ) : loading ? (
        <Loading />
      ) : (
        <>
          {[
            ...(data?.divisions || []),
            {
              id: null,
              name: 'Independent departments',
              description: 'Teams that work across the organization.',
            },
          ].map((division: Row) => {
            const deps = data?.departments.filter((d: Row) => d.division_id === division.id) || [];
            return deps.length ? (
              <section key={division.id ?? 'independent'} class="division-section">
                <div class="division-heading">
                  <span class="stat-icon plum">
                    <Icon name="organization" />
                  </span>
                  <div>
                    <h2>{division.name}</h2>
                    <p>{division.description}</p>
                  </div>
                  <Badge>{deps.length} departments</Badge>
                </div>
                <div class="card-grid">
                  {deps.map((d: Row) => (
                    <Panel key={d.id}>
                      <button
                        type="button"
                        class="department-card"
                        onClick={() => setDepartment(d)}
                      >
                        <div class="policy-top">
                          <span class="department-letter">{d.name[0]}</span>
                          <Icon name="arrow" size={18} />
                        </div>
                        <h3>{d.name}</h3>
                        <p>{d.description}</p>
                        <div class="department-card-bottom">
                          <span>
                            <Icon name="people" size={16} />
                            {d.member_count === undefined
                              ? 'View team'
                              : `${d.member_count} memberships`}
                          </span>
                          <small>Lead: {d.lead_name || 'Unassigned'}</small>
                        </div>
                      </button>
                    </Panel>
                  ))}
                </div>
              </section>
            ) : null;
          })}
          <div class="info-note">
            <Icon name="organization" />
            <span>
              Department membership and manager relationships are maintained separately. A team lead
              isn’t automatically everyone’s manager.
            </span>
          </div>
        </>
      )}
      {create && (
        <Modal title={`Add ${create}`} onClose={() => setCreate('')}>
          <Form
            fields={[
              { name: 'name', label: 'Name', required: true },
              ...(create === 'department'
                ? [
                    { name: 'slug', label: 'URL-friendly slug', required: true },
                    {
                      name: 'division_id',
                      label: 'Division',
                      type: 'select',
                      options: data?.divisions.map((d: Row) => ({ value: d.id, label: d.name })),
                    },
                  ]
                : []),
              { name: 'description', label: 'Description', type: 'textarea' },
            ]}
            submit={`Create ${create}`}
            onSubmit={async (b) => {
              if (b.division_id === '') b.division_id = null;
              await api(create === 'department' ? '/departments' : '/divisions', 'POST', b);
              setCreate('');
              refresh();
            }}
          />
        </Modal>
      )}
      {department && (
        <Modal title={department.name} onClose={() => setDepartment(null)} wide>
          <p class="modal-description">{department.description}</p>
          <DepartmentEditor department={department} />
          <p>
            <strong>Department lead:</strong> {department.lead_name || 'Unassigned'}
          </p>
          <div class="member-list">
            {members?.items.map((u: Row) => (
              <a key={u.id} class="person-cell" href={`/people?person=${u.id}`}>
                <Avatar name={u.display_name} />
                <span>
                  <strong>{u.display_name}</strong>
                  <small>
                    {u.level || 'Community'}
                    {u.primary_department_id === department.id ? ' · Primary department' : ''}
                  </small>
                </span>
                <Icon name="chevron" size={16} />
              </a>
            ))}
          </div>
        </Modal>
      )}
    </>
  );
}
