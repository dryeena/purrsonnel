import { useEffect, useRef, useState } from 'preact/hooks';

import { ErrorBox, Form, Icon, Modal } from '../../components/ui';
import { date } from '../../lib/date';
import { refresh } from '../../lib/events';
import { api } from '../../services/api-client';
import type { Row } from '../../types/api';
import type { BarcodeDetectorConstructor } from '../../types/browser';

export function Scanner({ onClose }: { onClose: () => void }) {
  const [idempotencyKey] = useState(() => crypto.randomUUID());
  const [raw, setRaw] = useState(new URLSearchParams(location.search).get('t') || '');
  const [entry, setEntry] = useState<Row | null>(null);
  const [error, setError] = useState('');
  const [token, setToken] = useState('');
  const [success, setSuccess] = useState(false);
  const [camera, setCamera] = useState(false);
  const video = useRef<HTMLVideoElement>(null);
  async function load(value: string) {
    setError('');
    try {
      let t = value.trim();
      if (t.includes('://')) {
        const url = new URL(t);
        if (url.origin !== location.origin)
          throw new Error('Use a Purrsonnel approval link from this site.');
        t = url.searchParams.get('t') || '';
      }
      if (!t) throw new Error('Paste an approval link or token.');
      setEntry(await api(`/volunteer/approval-preview?token=${encodeURIComponent(t)}`));
      setToken(t);
    } catch (e) {
      setError((e as Error).message);
    }
  }
  useEffect(() => {
    const initial = new URLSearchParams(location.search).get('t');
    if (initial) void load(initial);
  }, []);
  useEffect(() => {
    if (!camera) return;
    let stream: MediaStream | undefined;
    let stopped = false;
    let timer: ReturnType<typeof setTimeout>;
    const Detector = (window as Window & { BarcodeDetector?: BarcodeDetectorConstructor })
      .BarcodeDetector;
    if (!Detector) {
      setError('QR camera scanning is not supported by this browser. Paste the link below.');
      setCamera(false);
      return;
    }
    navigator.mediaDevices
      .getUserMedia({ video: { facingMode: 'environment' } })
      .then(async (s) => {
        stream = s;
        if (stopped) {
          s.getTracks().forEach((t) => t.stop());
          return;
        }
        if (video.current) {
          video.current.srcObject = s;
          await video.current.play();
        }
        const detector = new Detector({ formats: ['qr_code'] });
        const scan = async () => {
          if (stopped) return;
          try {
            if (video.current && video.current.readyState >= 2) {
              const codes = await detector.detect(video.current);
              if (codes[0]) {
                setRaw(codes[0].rawValue);
                setCamera(false);
                void load(codes[0].rawValue);
                return;
              }
            }
          } catch {
            setError('Camera could not read the code. Try the link instead.');
            setCamera(false);
            return;
          }
          timer = setTimeout(scan, 350);
        };
        void scan();
      })
      .catch(() => {
        setError('Camera access is unavailable. Paste the approval link below.');
        setCamera(false);
      });
    return () => {
      stopped = true;
      clearTimeout(timer);
      stream?.getTracks().forEach((t) => t.stop());
    };
  }, [camera]);
  return (
    <Modal title={success ? 'Decision recorded' : 'Review volunteer time'} onClose={onClose}>
      {success ? (
        <div class="success-box">
          <Icon name="check" /> The volunteer’s time entry has been updated.
        </div>
      ) : (
        <>
          {!entry && (
            <>
              <p class="modal-description">
                Scanning opens the entry for review. You’ll confirm the decision yourself.
              </p>
              <button type="button" class="button secondary" onClick={() => setCamera(!camera)}>
                <Icon name="qr" />
                {camera ? 'Stop camera' : 'Use camera'}
              </button>
              {camera && <video class="scanner-video" ref={video} playsInline muted />}
              <form
                class="form"
                onSubmit={(e) => {
                  e.preventDefault();
                  void load(raw);
                }}
              >
                <label class="field">
                  <span>Approval link or token</span>
                  <input value={raw} onInput={(e) => setRaw(e.currentTarget.value)} required />
                </label>
                <button type="submit" class="button primary">
                  Find entry
                  <Icon name="arrow" size={16} />
                </button>
              </form>
            </>
          )}
          {error && <ErrorBox message={error} />}{' '}
          {entry && (
            <>
              <div class="approval-summary">
                <h3>{entry.volunteer_name}</h3>
                <p>
                  {entry.department} · {date(entry.work_date)}
                </p>
                <strong class="approval-hours">
                  {(entry.minutes_worked / 60).toFixed(1)} hours
                </strong>
                <p>{entry.work_description}</p>
              </div>
              <Form
                fields={[
                  {
                    name: 'decision',
                    label: 'Your decision',
                    type: 'select',
                    required: true,
                    options: [
                      { value: 'approve', label: 'Approve hours' },
                      { value: 'deny', label: 'Reject hours' },
                    ],
                  },
                  {
                    name: 'note',
                    label: 'Decision note (required for rejection)',
                    type: 'textarea',
                    maxLength: 2000,
                  },
                  {
                    name: 'confirm',
                    label: 'I reviewed this entry and confirm this decision.',
                    type: 'checkbox',
                    required: true,
                  },
                ]}
                submit="Confirm decision"
                onSubmit={async (b) => {
                  await api('/volunteer/approval-decisions', 'POST', {
                    token,
                    decision: b.decision,
                    note: b.note,
                    idempotency_key: idempotencyKey,
                  });
                  setSuccess(true);
                  refresh();
                }}
              />
            </>
          )}
        </>
      )}
    </Modal>
  );
}
