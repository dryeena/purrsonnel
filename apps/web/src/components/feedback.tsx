import { useCallback, useEffect, useRef, useState } from 'preact/hooks';

import type { Notice } from '../lib/feedback';

import { Icon } from './ui';
interface Toast extends Notice {
  id: number;
}
export function Feedback() {
  const [notices, setNotices] = useState<Toast[]>([]);
  const sequence = useRef(0);
  const dismiss = useCallback((id: number) => {
    setNotices((current) => current.filter((notice) => notice.id !== id));
  }, []);
  useEffect(() => {
    const listener = (event: Event) => {
      const detail = (event as CustomEvent<Notice>).detail;
      if (!detail || typeof detail.message !== 'string' || !detail.message.trim()) return;
      const notice = {
        id: ++sequence.current,
        message: detail.message.slice(0, 240),
        tone: detail.tone === 'info' ? ('info' as const) : ('success' as const),
      };
      setNotices((current) => [...current.slice(-2), notice]);
    };
    window.addEventListener('purrsonnel:notice', listener);
    return () => window.removeEventListener('purrsonnel:notice', listener);
  }, []);
  return (
    <section class="feedback-stack" aria-label="Action feedback">
      {notices.map((notice) => (
        <ToastMessage key={notice.id} notice={notice} onDismiss={dismiss} />
      ))}
    </section>
  );
}
function ToastMessage({ notice, onDismiss }: { notice: Toast; onDismiss: (id: number) => void }) {
  const [paused, setPaused] = useState(false);
  useEffect(() => {
    if (paused) return;
    const timer = setTimeout(() => onDismiss(notice.id), 6500);
    return () => clearTimeout(timer);
  }, [paused, notice.id, onDismiss]);
  return (
    <div
      class={`feedback-toast ${notice.tone}`}
      onMouseEnter={() => setPaused(true)}
      onMouseLeave={() => setPaused(false)}
      onFocusIn={() => setPaused(true)}
      onFocusOut={() => setPaused(false)}
    >
      <span class="feedback-symbol">
        <Icon name={notice.tone === 'success' ? 'check' : 'bell'} size={18} />
      </span>
      <p role="status">{notice.message}</p>
      <button
        type="button"
        class="icon-button"
        aria-label="Dismiss message"
        onClick={() => onDismiss(notice.id)}
      >
        <Icon name="close" size={16} />
      </button>
    </div>
  );
}
