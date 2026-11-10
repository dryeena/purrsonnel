import { useState } from 'preact/hooks';

import {
  Action,
  Avatar,
  Badge,
  ErrorBox,
  Form,
  Icon,
  Loading,
  Modal,
  Status,
} from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { downloadJson } from '../../lib/exports';
import { can, global } from '../../lib/permissions';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';
import { CustomProfileFields, PersonAssignments } from '../configuration';

export function Person({ id, onClose }: { id: string; onClose: () => void }) {
  const me = useAuth();
  const { data: u, error, loading } = useData<Row>(`/users/${id}`);
  const { data: org } = useData<Row>('/organization/tree');
  const { data: skills } = useData<Row[]>('/skills');
  const { data: directs } = useData<Row[]>(`/users/${id}/directs`);
  const [edit, setEdit] = useState(false);
  const [year, setYear] = useState(false);
  const [skill, setSkill] = useState(false);
  return (
    <Modal title="Person details" onClose={onClose} wide>
      {loading ? (
        <Loading />
      ) : error ? (
        <ErrorBox message={error} />
      ) : (
        u && (
          <>
            <div class="person-hero">
              <Avatar name={u.display_name} />
              <div>
                <h2>{u.display_name}</h2>
                <p>{u.myfwa_id || 'Identity pending'}</p>
              </div>
              <Status value={u.status} />
            </div>
            <div class="detail-grid">
              <div>
                <span>Personnel level</span>
                <strong>{u.level || 'Community member'}</strong>
              </div>
              <div>
                <span>Volunteer eligibility</span>
                <strong>{u.volunteer_eligible ? 'Eligible' : 'Not currently eligible'}</strong>
              </div>
              <div>
                <span>Primary department</span>
                <strong>{u.department || 'Unassigned'}</strong>
              </div>
              <div>
                <span>Verified years worked</span>
                <strong>{u.verified_years} distinct years</strong>
              </div>
            </div>
            <div class="detail-section">
              <h3>Department memberships</h3>
              <div class="tag-stack">
                {u.departments.map((d: Row) => (
                  <Badge key={d.id} tone="plum">
                    {d.name}
                    {d.id === u.primary_department_id ? ' · primary' : ''}
                  </Badge>
                ))}
              </div>
            </div>
            <div class="detail-section">
              <h3>Service, one year at a time</h3>
              <p class="muted">Only explicitly verified years count. Gaps are welcome.</p>
              <div class="tag-stack">
                {u.service_years.map((y: number) => (
                  <span key={y} class="year-chip">
                    {y}
                    {can(me, 'people.edit') && (
                      <Action
                        className="icon-button"
                        action={async () => {
                          await api(`/users/${id}/service-years/${y}`, 'DELETE');
                          refresh();
                        }}
                      >
                        ×
                      </Action>
                    )}
                  </span>
                ))}
                {can(me, 'people.edit') && (
                  <button type="button" class="text-link" onClick={() => setYear(!year)}>
                    + Record a year
                  </button>
                )}
              </div>
              {year && (
                <Form
                  fields={[
                    {
                      name: 'year',
                      label: 'Year worked',
                      type: 'number',
                      required: true,
                      min: 1990,
                      max: 2200,
                      value: new Date().getFullYear(),
                    },
                  ]}
                  submit="Record verified year"
                  onSubmit={async (b) => {
                    await api(`/users/${id}/service-years/${b.year}`, 'PUT', {
                      verification_status: 'verified',
                    });
                    setYear(false);
                    refresh();
                  }}
                />
              )}
            </div>
            <div class="detail-section">
              <h3>Skills</h3>
              <div class="tag-stack">
                {u.skills.map((s: string) => (
                  <Badge key={s}>{s}</Badge>
                ))}
                {can(me, 'people.edit') && (
                  <button type="button" class="text-link" onClick={() => setSkill(!skill)}>
                    + Add skill
                  </button>
                )}
              </div>
              {skill && (
                <Form
                  fields={[
                    {
                      name: 'skill',
                      label: 'Skill',
                      type: 'select',
                      required: true,
                      options: skills?.map((s) => ({ value: s.id, label: s.name })),
                    },
                  ]}
                  submit="Add skill"
                  onSubmit={async (b) => {
                    await api(`/users/${id}/skills/${b.skill}`, 'PUT');
                    setSkill(false);
                    refresh();
                  }}
                />
              )}
            </div>
            <div class="detail-grid">
              <div>
                <span>Reports to</span>
                <strong>
                  {u.managers.map((m: Row) => m.display_name).join(', ') || 'No manager assigned'}
                </strong>
              </div>
              <div>
                <span>Direct reports</span>
                <strong>
                  {directs?.map((d) => d.display_name).join(', ') || 'No direct reports'}
                </strong>
              </div>
            </div>
            <PersonAssignments person={u} />
            {global(me, 'people.read_sensitive') && (
              <Action
                action={async () => {
                  downloadJson(`purrsonnel-person-${id}.json`, await api(`/users/${id}/export`));
                }}
              >
                Export person record
              </Action>
            )}
            {!can(me, 'people.edit') && <CustomProfileFields personId={id} />}
            {can(me, 'people.edit') && (
              <button type="button" class="button secondary" onClick={() => setEdit(!edit)}>
                {edit ? 'Close profile editor' : 'Edit profile'}
                <Icon name="arrow" size={15} />
              </button>
            )}
            {edit && (
              <div class="detail-section">
                <h3>Edit profile</h3>
                <Form
                  fields={[
                    {
                      name: 'display_name',
                      label: 'Display name',
                      required: true,
                      value: u.display_name,
                    },
                    {
                      name: 'status',
                      label: 'Status',
                      type: 'select',
                      required: true,
                      value: u.status,
                      options: ['active', 'inactive', 'former'].map((value) => ({
                        value,
                        label: value,
                      })),
                    },
                    {
                      name: 'volunteer_eligible',
                      label: 'Eligible to volunteer',
                      type: 'checkbox',
                      value: u.volunteer_eligible,
                    },
                    ...(global(me, 'people.edit')
                      ? [
                          {
                            name: 'personnel_level_id',
                            label: 'Personnel level',
                            type: 'select',
                            value: u.personnel_level_id,
                            options: org?.levels.map((l: Row) => ({ value: l.id, label: l.label })),
                          },
                        ]
                      : []),
                  ]}
                  onSubmit={async (b) => {
                    if (!b.personnel_level_id) delete b.personnel_level_id;
                    await api(`/users/${id}`, 'PATCH', b);
                    refresh();
                  }}
                />
                <CustomProfileFields personId={id} editing onSaved={() => setEdit(false)} />
              </div>
            )}
          </>
        )
      )}
    </Modal>
  );
}
