import { useState } from 'preact/hooks';

import { Action, Badge, Form, Icon, Panel } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { useAuth } from '../../lib/auth';
import { refresh } from '../../lib/events';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function PermissionManager() {
  const me = useAuth();
  const { data, error } = useData<Row>('/permission-grants');
  const { data: org } = useData<Row>('/organization/tree');
  const [open, setOpen] = useState(false);
  const [scope, setScope] = useState('global');
  return (
    <Panel
      title="Permission assignments"
      description="Dedicated, audited access changes. Rank does not grant permissions."
      action={
        <button type="button" class="text-link" onClick={() => setOpen(!open)}>
          + Grant permission
        </button>
      }
    >
      {error && <p class="error-box">{error}</p>}
      {open && (
        <div class="setting-content">
          <label class="field">
            <span>Scope</span>
            <select value={scope} onChange={(e) => setScope(e.currentTarget.value)}>
              <option value="global">Global</option>
              <option value="department">Department</option>
              <option value="division">Division</option>
            </select>
          </label>
          <Form
            key={scope}
            fields={[
              {
                name: 'user_id',
                label: 'Person',
                type: 'select',
                required: true,
                options: me.identities.map((u) => ({ value: u.id, label: u.display_name })),
              },
              {
                name: 'permission',
                label: 'Permission',
                type: 'select',
                required: true,
                options: data?.permissions.map((p: string) => ({ value: p, label: p })),
              },
              ...(scope === 'global'
                ? []
                : [
                    {
                      name: 'scope_id',
                      label: scope === 'department' ? 'Department' : 'Division',
                      type: 'select',
                      required: true,
                      options: org?.[scope === 'department' ? 'departments' : 'divisions'].map(
                        (d: Row) => ({ value: d.id, label: d.name }),
                      ),
                    },
                  ]),
            ]}
            submit="Grant permission"
            onSubmit={async (b) => {
              await api('/permission-grants', 'POST', {
                ...b,
                scope_type: scope,
                scope_id: b.scope_id || null,
              });
              setOpen(false);
              refresh();
            }}
          />
        </div>
      )}
      <div class="permission-list">
        {data?.items.map((g: Row) => (
          <div key={g.id}>
            <div>
              <strong>{g.display_name}</strong>
              <small>{g.permission}</small>
            </div>
            <Badge>{g.scope_type}</Badge>
            <Action
              className="icon-button"
              action={async () => {
                await api(`/permission-grants/${g.id}`, 'DELETE');
                refresh();
              }}
            >
              <Icon name="close" size={14} />
              <span class="sr-only">
                Revoke {g.permission} from {g.display_name}
              </span>
            </Action>
          </div>
        ))}
      </div>
    </Panel>
  );
}
