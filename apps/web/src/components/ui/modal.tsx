import { useCallback, useEffect, useId, useRef } from 'preact/hooks';

import type { ComponentChildren } from 'preact';

import { Icon } from './icon';

const modalStack: symbol[] = [];

export function Modal({
  title,
  children,
  onClose,
  wide = false,
}: {
  title: string;
  children: ComponentChildren;
  onClose: () => void;
  wide?: boolean;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const titleId = useId();
  const onCloseRef = useRef(onClose);
  onCloseRef.current = onClose;
  const close = useCallback(() => {
    if (ref.current?.querySelector('form[aria-busy="true"]')) return;
    if (
      ref.current?.querySelector('form[data-dirty="true"]') &&
      !window.confirm('Discard the unsaved changes in this dialog?')
    )
      return;
    onCloseRef.current();
  }, []);
  useEffect(() => {
    const key = Symbol('dialog');
    modalStack.push(key);
    const old = document.activeElement as HTMLElement | null;
    const previousOverflow = document.body.style.overflow;
    document.body.style.overflow = 'hidden';
    ref.current?.focus();
    const handle = (e: KeyboardEvent) => {
      if (modalStack[modalStack.length - 1] !== key) return;
      if (e.key === 'Escape') {
        e.preventDefault();
        close();
      }
      if (e.key === 'Tab') {
        const controls = ref.current?.querySelectorAll<HTMLElement>(
          'button:not(:disabled),a[href],input:not(:disabled),textarea:not(:disabled),select:not(:disabled),[tabindex="0"]',
        );
        if (!controls?.length) return;
        const first = controls[0],
          last = controls[controls.length - 1];
        if (
          e.shiftKey &&
          (document.activeElement === first || document.activeElement === ref.current)
        ) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    };
    document.addEventListener('keydown', handle);
    return () => {
      modalStack.splice(modalStack.indexOf(key), 1);
      document.body.style.overflow = modalStack.length ? 'hidden' : previousOverflow;
      document.removeEventListener('keydown', handle);
      if (old?.isConnected) old.focus();
    };
  }, [close]);
  return (
    <div
      class="modal-backdrop"
      onClick={(e) => {
        if (e.target === e.currentTarget) close();
      }}
    >
      <div
        ref={ref}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        tabIndex={-1}
        class={`modal ${wide ? 'wide' : ''}`}
      >
        <div class="modal-header">
          <h2 id={titleId}>{title}</h2>
          <button type="button" class="icon-button" aria-label="Close dialog" onClick={close}>
            <Icon name="close" />
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}
