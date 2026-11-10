import { useState } from 'preact/hooks';

import { humanize } from '../../lib/text';
import type { Row } from '../../types/api';

export function describeFilter(value: Row): string {
  if (['and', 'or'].includes(value.op))
    return value.args.length
      ? value.args.map(describeFilter).join(` ${value.op.toUpperCase()} `)
      : 'All people in your scope';
  if (value.op === 'not') return `NOT (${describeFilter(value.arg)})`;
  return `${humanize(value.field)} ${value.op} ${String(value.value ?? '')}`;
}

export function FilterBuilder({ value, onChange }: { value: Row; onChange: (v: Row) => void }) {
  const [field, setField] = useState('status');
  const [match, setMatch] = useState('active');
  const [op, setOp] = useState('eq');
  return (
    <div class="filter-builder">
      <div class="inline-fields">
        <strong>Match</strong>
        <select
          aria-label="Match filter group"
          value={value.op}
          onChange={(e) => onChange({ ...value, op: e.currentTarget.value })}
        >
          <option value="and">all conditions (AND)</option>
          <option value="or">any condition (OR)</option>
        </select>
      </div>
      {value.args.map((f: Row, i: number) => (
        <div key={i} class="filter-chip">
          {describeFilter(f)}
          <button
            type="button"
            class="icon-button"
            aria-label={`Remove filter ${i + 1}`}
            onClick={() =>
              onChange({ ...value, args: value.args.filter((_: Row, n: number) => n !== i) })
            }
          >
            ×
          </button>
        </div>
      ))}
      <div class="inline-fields">
        <select
          aria-label="Filter field"
          value={field}
          onChange={(e) => setField(e.currentTarget.value)}
        >
          {['status', 'display_name', 'department', 'level', 'myfwa_id', 'volunteer_eligible'].map(
            (f) => (
              <option key={f} value={f}>
                {humanize(f)}
              </option>
            ),
          )}
        </select>
        <select
          aria-label="Filter comparison"
          value={op}
          onChange={(e) => setOp(e.currentTarget.value)}
        >
          <option value="eq">equals</option>
          <option value="contains">contains</option>
        </select>
        <input
          aria-label="Filter value"
          value={match}
          onInput={(e) => setMatch(e.currentTarget.value)}
          placeholder={field === 'volunteer_eligible' ? 'true or false' : 'Value'}
        />
        <button
          type="button"
          class="button secondary"
          onClick={() =>
            onChange({
              ...value,
              args: [
                ...value.args,
                { op, field, value: field === 'volunteer_eligible' ? match === 'true' : match },
              ],
            })
          }
        >
          Add
        </button>
      </div>
      <p class="small-note">An empty group includes all people in your authorized scope.</p>
    </div>
  );
}
