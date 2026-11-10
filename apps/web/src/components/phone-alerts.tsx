import { useEffect, useState } from 'preact/hooks';

import { useData } from '../hooks/use-data';
import { disablePush, enablePush, pushSupportMessage } from '../lib/push';
import { api } from '../services/api-client';
import type { Row } from '../types/api';

import { AlertPreferences } from './alert-preferences';
import { InstallApp } from './install';
import { Action, Badge, Icon } from './ui';
export function PhoneAlerts() {
  const { data, error, reload } = useData<Row>('/push/config');
  const [enabled, setEnabled] = useState(false);
  const [checked, setChecked] = useState(false);
  const [preferences, setPreferences] = useState(false);
  const limitation = pushSupportMessage();
  useEffect(() => {
    let alive = true;
    if (!('serviceWorker' in navigator)) {
      setChecked(true);
      return;
    }
    navigator.serviceWorker
      .getRegistration('/')
      .then((r) => r?.pushManager?.getSubscription())
      .then(async (s) => {
        const active = s
          ? (
              await api<{ active: boolean }>('/push/subscriptions/status', 'POST', {
                endpoint: s.endpoint,
              })
            ).active
          : false;
        if (alive) {
          setEnabled(active);
          setChecked(true);
        }
      })
      .catch(() => {
        if (alive) setChecked(true);
      });
    return () => {
      alive = false;
    };
  }, []);
  return (
    <>
      <section class="phone-alerts">
        <span class="stat-icon plum">
          <Icon name="bell" />
        </span>
        <div>
          <h2>
            Get a heads-up on your phone{' '}
            {enabled && (
              <Badge tone={data?.enabled && !limitation ? 'green' : 'amber'}>
                {data?.enabled && !limitation ? 'Device subscribed' : 'Needs attention'}
              </Badge>
            )}
          </h2>
          <p>
            {error
              ? 'Alert settings could not be loaded. Your inbox is still available.'
              : !data
                ? 'Checking phone alert settings…'
                : !data.enabled
                  ? 'Phone alerts are not available for this workspace yet. You can still send and receive requests here.'
                  : limitation ||
                    'Choose to receive a notification when someone sends you or your team a request.'}
          </p>
          <small>Phone preferences and device settings control which alerts can buzz.</small>
          <button
            type="button"
            class="text-link alert-preferences-link"
            onClick={() => setPreferences(true)}
          >
            <Icon name="settings" size={15} />
            Alert preferences & quiet hours
          </button>
          {error && (
            <button type="button" class="text-link" onClick={reload}>
              Try again
            </button>
          )}
          {limitation && <InstallApp />}
        </div>
        {enabled ? (
          <Action
            action={async () => {
              await disablePush();
              setEnabled(false);
            }}
          >
            Turn alerts off
          </Action>
        ) : data?.enabled && !limitation && checked ? (
          <Action
            className="button secondary"
            action={async () => {
              await enablePush(data.public_key);
              setEnabled(true);
            }}
          >
            Enable phone alerts
          </Action>
        ) : null}
      </section>
      {preferences && <AlertPreferences onClose={() => setPreferences(false)} />}
    </>
  );
}
