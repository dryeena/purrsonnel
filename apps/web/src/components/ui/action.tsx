import { useState } from 'preact/hooks';

import type { ComponentChildren } from 'preact';

import { notify } from '../../lib/feedback';

import { ErrorBox } from './primitives';

export function Action({
  children,
  action,
  className = 'button secondary',
  successMessage,
  ariaLabel,
  disabled = false,
}: {
  children: ComponentChildren;
  action: () => Promise<void>;
  className?: string;
  successMessage?: string;
  ariaLabel?: string;
  disabled?: boolean;
}) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  return (
    <>
      <button
        type="button"
        class={className}
        aria-label={ariaLabel}
        aria-busy={busy}
        disabled={busy || disabled}
        onClick={async () => {
          if (busy || disabled) return;
          setBusy(true);
          setError('');
          try {
            await action();
            if (successMessage) notify(successMessage);
          } catch (e) {
            setError((e as Error).message);
          } finally {
            setBusy(false);
          }
        }}
      >
        {busy ? (
          <>
            <span class="spinner button-spinner" aria-hidden="true" />
            Working…
          </>
        ) : (
          children
        )}
      </button>
      {error && <ErrorBox message={error} />}
    </>
  );
}
