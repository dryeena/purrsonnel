import { useState } from 'preact/hooks';

import { Form, Panel } from '../../components/ui';
import { useData } from '../../hooks/use-data';
import { refresh } from '../../lib/events';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';

export function LevelManager() {
  const { data: org } = useData<Row>('/organization/tree');
  const [add, setAdd] = useState(false);
  return (
    <Panel
      title="Position level mapping"
      description="Other must have an explicit mapping before hiring."
    >
      <div class="setting-content">
        {org?.position_levels.map((l: Row) => (
          <div key={l.id} class="mapping-row">
            <strong>{l.label}</strong>
            <Form
              fields={[
                {
                  name: 'personnel_level_id',
                  label: `Personnel mapping for ${l.label}`,
                  type: 'select',
                  required: true,
                  value: l.personnel_level_id,
                  options: org.levels.map((p: Row) => ({ value: p.id, label: p.label })),
                },
              ]}
              submit="Update mapping"
              onSubmit={async (b) => {
                await api(`/position-levels/${l.id}`, 'PATCH', b);
                refresh();
              }}
            />
          </div>
        ))}
        <button type="button" class="text-link" onClick={() => setAdd(!add)}>
          + Add personnel level
        </button>
        {add && (
          <Form
            fields={[
              { name: 'label', label: 'Level label', required: true },
              { name: 'slug', label: 'Stable slug', required: true },
              {
                name: 'rank',
                label: 'Ordinal rank',
                type: 'number',
                min: 1,
                max: 1000,
                required: true,
              },
            ]}
            submit="Create level"
            onSubmit={async (b) => {
              await api('/levels', 'POST', b);
              setAdd(false);
              refresh();
            }}
          />
        )}
      </div>
    </Panel>
  );
}
