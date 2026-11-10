import { useEffect, useState } from 'preact/hooks';

import QRCode from 'qrcode';

import { Action, ErrorBox, Loading, Modal } from '../../components/ui';
import type { Row } from '../../types/api';

export function QrModal({ data, onClose }: { data: Row; onClose: () => void }) {
  const [image, setImage] = useState('');
  const [error, setError] = useState('');
  const [copied, setCopied] = useState(false);
  const [now, setNow] = useState(Date.now());
  useEffect(() => {
    QRCode.toDataURL(data.url, {
      width: 288,
      margin: 2,
      color: { dark: '#482348', light: '#ffffff' },
      errorCorrectionLevel: 'M',
    })
      .then(setImage)
      .catch(() => setError('Could not generate the QR image. Use the link below.'));
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, [data.url]);
  const remaining = data.expires_at
    ? Math.max(0, Math.ceil((new Date(data.expires_at).getTime() - now) / 1000))
    : null;
  return (
    <Modal
      title={data.shortcut ? 'A welcome starts with a link' : 'Volunteer time approval'}
      onClose={onClose}
    >
      <p class="modal-description">
        {data.shortcut
          ? 'This code opens the volunteer page. It contains no approval credential.'
          : 'Ask an authorized staff member to scan, review your entry, and confirm their decision.'}
      </p>
      <div class="qr-image">
        {image ? (
          <img
            src={image}
            alt={
              data.shortcut
                ? 'QR link to volunteer page'
                : 'Short-lived QR link to review your time entry'
            }
          />
        ) : (
          <Loading />
        )}
      </div>
      {error && <ErrorBox message={error} />}
      <label class="field">
        <span>Shareable link</span>
        <input readOnly value={data.url} onFocus={(e) => e.currentTarget.select()} />
      </label>
      {remaining !== null && (
        <p class={remaining === 0 ? 'error-text' : 'muted'} role="status">
          {remaining === 0
            ? 'This code has expired. Close and generate a new code.'
            : `Expires in ${Math.floor(remaining / 60)}:${String(remaining % 60).padStart(2, '0')}. Regenerating invalidates the previous code.`}
        </p>
      )}
      <Action
        action={async () => {
          await navigator.clipboard.writeText(data.url);
          setCopied(true);
        }}
      >
        {copied ? 'Link copied' : 'Copy link'}
      </Action>
    </Modal>
  );
}
