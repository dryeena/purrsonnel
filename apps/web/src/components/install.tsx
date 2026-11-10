import { useEffect, useState } from 'preact/hooks';

import { clearInstall, isAppleMobile, isStandalone, pendingInstall } from '../lib/pwa';

import { Action, Icon, Modal } from './ui';
export function InstallApp() {
  const [open, setOpen] = useState(false);
  const [, changed] = useState(0);
  useEffect(() => {
    const listener = () => changed((value) => value + 1);
    window.addEventListener('purrsonnel:install', listener);
    const media = matchMedia('(display-mode: standalone)');
    media.addEventListener('change', listener);
    return () => {
      window.removeEventListener('purrsonnel:install', listener);
      media.removeEventListener('change', listener);
    };
  }, []);
  if (isStandalone()) return null;
  return (
    <>
      <button type="button" class="install-link" onClick={() => setOpen(true)}>
        <Icon name="download" size={16} />
        <span>Add to home screen</span>
      </button>
      {open && (
        <Modal title="Keep Purrsonnel close" onClose={() => setOpen(false)}>
          <div class="install-identity">
            <img src="/branding/icon-192.png" width="60" height="60" alt="" />
            <div>
              <h3>Purrsonnel</h3>
              <p>Your team, just a tap away.</p>
            </div>
          </div>
          <p class="modal-description">
            Open your workspace from your home screen in its own app window. Your sign-in and
            permissions work the same way.
          </p>
          {pendingInstall() ? (
            <Action
              className="button primary"
              action={async () => {
                const prompt = pendingInstall();
                if (!prompt) return;
                await prompt.prompt();
                const choice = await prompt.userChoice;
                clearInstall();
                if (choice.outcome === 'accepted') setOpen(false);
              }}
            >
              Install Purrsonnel
              <Icon name="arrow" size={16} />
            </Action>
          ) : isAppleMobile() ? (
            <ol class="install-steps">
              <li>Open this site in Safari.</li>
              <li>
                Tap <strong>Share</strong>, then <strong>Add to Home Screen</strong>.
              </li>
              <li>
                Keep <strong>Open as Web App</strong> enabled if shown, then tap{' '}
                <strong>Add</strong>.
              </li>
            </ol>
          ) : (
            <ol class="install-steps">
              <li>Open your browser menu.</li>
              <li>
                Choose <strong>Install Purrsonnel</strong>, <strong>Install app</strong>, or{' '}
                <strong>Add to Home screen</strong>, if available.
              </li>
              <li>Open the new Purrsonnel icon to return to your workspace.</li>
            </ol>
          )}
          <p class="small-note">
            Phone alerts are optional. You can enable them from Requests after opening the installed
            app.
          </p>
        </Modal>
      )}
    </>
  );
}
