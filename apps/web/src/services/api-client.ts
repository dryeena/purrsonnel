import type { Row } from '../types/api';

let csrf = '';

export function setCsrf(value: string) {
  csrf = value;
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}

export async function api<T = Row>(path: string, method = 'GET', body?: unknown): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`/api/v1${path}`, {
      method,
      credentials: 'same-origin',
      headers: { 'Content-Type': 'application/json', ...(csrf ? { 'X-CSRF-Token': csrf } : {}) },
      ...(body === undefined ? {} : { body: JSON.stringify(body) }),
    });
  } catch {
    throw new Error(
      navigator.onLine
        ? 'Purrsonnel could not be reached. Your action was not confirmed; check your connection and try again.'
        : 'You’re offline. Reconnect to load your workspace or save changes.',
    );
  }
  const result = await response.json().catch(() => ({}));
  if (!response.ok)
    throw new ApiError(response.status, result.detail || `Request failed (${response.status})`);
  return result as T;
}
