import { useEffect, useId, useRef, useState } from 'preact/hooks';

import type { ComponentChildren } from 'preact';

import { trackUnsaved } from '../../lib/unsaved';
import type { Row } from '../../types/api';

import { Icon } from './icon';
import { ErrorBox } from './primitives';

export interface FieldSpec {
  name: string;
  label: string;
  type?: string;
  required?: boolean;
  options?: { value: string; label: string }[];
  value?: unknown;
  help?: string;
  min?: number;
  max?: number;
  maxLength?: number;
  placeholder?: string;
}

export function FormField({ field: f, formId }: { field: FieldSpec; formId: string }) {
  const controlId = `${formId}-${f.name}`;
  const [count, setCount] = useState(String(f.value ?? '').length);
  const countable = ['text', 'textarea'].includes(f.type || 'text') && f.maxLength !== undefined;
  const describedBy =
    [f.help ? `${controlId}-hint` : null, countable ? `${controlId}-count` : null]
      .filter(Boolean)
      .join(' ') || undefined;
  const attributes = {
    id: controlId,
    name: f.name,
    required: f.required,
    'aria-describedby': describedBy,
  };
  const caption = (
    <>
      {f.label}
      {f.required && <b aria-hidden="true"> *</b>}
    </>
  );
  const hints = (
    <>
      {f.help && (
        <small id={`${controlId}-hint`} class="field-hint">
          {f.help}
        </small>
      )}
      {countable && (
        <small
          id={`${controlId}-count`}
          class={`field-counter ${count === f.maxLength ? 'at-limit' : ''}`}
        >
          {count} / {f.maxLength} characters
        </small>
      )}
    </>
  );
  if (f.type === 'checkbox')
    return (
      <div class="checkbox-field">
        <label class="check-field" htmlFor={controlId}>
          <input {...attributes} type="checkbox" defaultChecked={!!f.value} />
          <span>{caption}</span>
        </label>
        {hints}
      </div>
    );
  if (f.type === 'signature') return <SignatureField field={f} controlId={controlId} />;
  return (
    <div class="field">
      <label class="field-label" htmlFor={controlId}>
        {caption}
      </label>
      {f.type === 'textarea' ? (
        <textarea
          {...attributes}
          defaultValue={f.value == null ? undefined : String(f.value)}
          placeholder={f.placeholder}
          maxLength={f.maxLength ?? 4000}
          rows={4}
          onInput={(e) => setCount(e.currentTarget.value.length)}
        />
      ) : f.type === 'multi_select' ? (
        <select {...attributes} multiple size={Math.min(f.options?.length || 3, 5)}>
          {f.options?.map((o) => (
            <option
              key={o.value}
              value={o.value}
              selected={Array.isArray(f.value) && f.value.includes(o.value)}
            >
              {o.label}
            </option>
          ))}
        </select>
      ) : f.type === 'select' ? (
        <select {...attributes}>
          {f.required ? (
            <option value="" disabled selected={f.value == null || f.value === ''}>
              Choose an option…
            </option>
          ) : (
            <option value="" selected={f.value == null || f.value === ''}>
              None
            </option>
          )}
          {f.options?.map((o) => (
            <option key={o.value} value={o.value} selected={String(f.value) === o.value}>
              {o.label}
            </option>
          ))}
        </select>
      ) : (
        <input
          {...attributes}
          type={f.type || 'text'}
          defaultValue={f.value == null ? undefined : String(f.value)}
          placeholder={f.placeholder}
          min={f.min}
          max={f.max}
          maxLength={f.maxLength ?? 200}
          onInput={(e) => setCount(e.currentTarget.value.length)}
        />
      )}
      {hints}
    </div>
  );
}

function SignatureField({ field, controlId }: { field: FieldSpec; controlId: string }) {
  const [points, setPoints] = useState<string[]>([]);
  const drawing = useRef(false);
  const point = (event: PointerEvent) => {
    const box = (event.currentTarget as SVGSVGElement).getBoundingClientRect();
    return `${Math.round(((event.clientX - box.left) / box.width) * 600)},${Math.round(((event.clientY - box.top) / box.height) * 160)}`;
  };
  const d = points.length ? `M ${points.join(' L ')}` : '';
  const value = d
    ? `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 160"><path d="${d}" fill="none" stroke="black" stroke-width="3" stroke-linecap="round"/></svg>`
    : '';
  return (
    <div class="field signature-field">
      <label class="field-label" htmlFor={controlId}>
        {field.label}
        {field.required && <b aria-hidden="true"> *</b>}
      </label>
      <svg
        id={controlId}
        class="signature-pad"
        viewBox="0 0 600 160"
        role="img"
        aria-label="Draw your signature"
        onPointerDown={(e) => {
          drawing.current = true;
          e.currentTarget.setPointerCapture(e.pointerId);
          setPoints([point(e)]);
        }}
        onPointerMove={(e) => {
          if (drawing.current) setPoints((prior) => [...prior, point(e)]);
        }}
        onPointerUp={() => {
          drawing.current = false;
        }}
      >
        <path d={d} fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
      </svg>
      <input type="hidden" name={field.name} value={value} />
      <button type="button" class="text-link" onClick={() => setPoints([])}>
        Clear signature
      </button>
      <small class="field-hint">Draw your signature with a mouse, trackpad, or touch.</small>
    </div>
  );
}

export function Form({
  fields,
  onSubmit,
  submit = 'Save changes',
  children,
  draft = false,
}: {
  fields: FieldSpec[];
  onSubmit: (values: Row) => Promise<void>;
  submit?: string;
  children?: ComponentChildren;
  draft?: boolean;
}) {
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [dirty, setDirty] = useState(false);
  const [saved, setSaved] = useState(false);
  const [online, setOnline] = useState(navigator.onLine);
  const key = useRef(Symbol('form'));
  const formId = useId();
  const errorRef = useRef<HTMLDivElement>(null);
  const markDirty = () => {
    setDirty(true);
    setSaved(false);
    trackUnsaved(key.current, true);
  };
  useEffect(() => {
    const formKey = key.current;
    const update = () => setOnline(navigator.onLine);
    window.addEventListener('online', update);
    window.addEventListener('offline', update);
    return () => {
      trackUnsaved(formKey, false);
      window.removeEventListener('online', update);
      window.removeEventListener('offline', update);
    };
  }, []);
  useEffect(() => {
    if (error) errorRef.current?.focus();
  }, [error]);
  return (
    <form
      class="form"
      noValidate={draft}
      data-dirty={dirty ? 'true' : undefined}
      aria-busy={busy}
      onInput={markDirty}
      onChange={markDirty}
      onSubmit={async (e) => {
        e.preventDefault();
        if (busy) return;
        if (!navigator.onLine) {
          setError('You’re offline. Reconnect before saving; your entries are still here.');
          return;
        }
        const fd = new FormData(e.currentTarget);
        const values: Row = {};
        for (const f of fields) {
          if (f.type === 'checkbox') values[f.name] = fd.has(f.name);
          else if (f.type === 'multi_select') values[f.name] = fd.getAll(f.name).map(String);
          else if (f.type === 'number')
            values[f.name] = fd.get(f.name) === '' ? null : Number(fd.get(f.name));
          else values[f.name] = String(fd.get(f.name) ?? '');
        }
        setError('');
        setBusy(true);
        setSaved(false);
        try {
          await onSubmit(values);
          setDirty(false);
          trackUnsaved(key.current, false);
          setSaved(true);
        } catch (e) {
          setError((e as Error).message);
        } finally {
          setBusy(false);
        }
      }}
    >
      {draft && (
        <p class="draft-note">
          <Icon name="terms" size={16} />
          You can save a draft with unanswered questions.
        </p>
      )}
      <fieldset class="form-fields" disabled={busy}>
        {fields.map((f) => (
          <FormField key={f.name} field={f} formId={formId} />
        ))}
      </fieldset>
      {children}
      {error && (
        <div ref={errorRef} class="form-error" tabIndex={-1}>
          <ErrorBox message={error} />
        </div>
      )}
      <div class="form-footer">
        {saved && (
          <span class="saved-note" role="status">
            <Icon name="check" size={15} />
            Saved
          </span>
        )}
        {!online && (
          <span class="small-note" role="status">
            Reconnect to save.
          </span>
        )}
        <button type="submit" class="button primary" disabled={busy || !online}>
          {busy ? (
            <>
              <span class="spinner button-spinner" aria-hidden="true" />
              Saving…
            </>
          ) : (
            <>
              {submit}
              <Icon name="arrow" size={16} />
            </>
          )}
        </button>
      </div>
    </form>
  );
}
