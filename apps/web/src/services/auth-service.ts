import type { Identity } from '../types/api';

import { api, setCsrf } from './api-client';

export async function authenticate(): Promise<Identity> {
  const result = await api('/auth/dev/bootstrap', 'POST');
  setCsrf(result.csrf_token);
  const identity = await api<Identity>('/auth/me');
  setCsrf(identity.csrf_token);
  return identity;
}
