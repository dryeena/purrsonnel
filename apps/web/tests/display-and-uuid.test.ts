import { describe, expect, it, vi } from 'vitest';

import { humanize } from '../src/lib/text';
import { uuid } from '../src/lib/uuid';

describe('display text', () => {
  it('converts stored keys into readable labels', () => {
    expect(humanize('pending_review')).toBe('Pending review');
    expect(humanize('department.lead')).toBe('Department lead');
  });

  it('keeps empty display values empty', () => {
    expect(humanize('')).toBe('');
  });
});

describe('uuid', () => {
  it('uses the browser UUID generator when it is available', () => {
    const randomUUID = vi.fn(() => 'preferred-id');
    vi.stubGlobal('crypto', { randomUUID });
    expect(uuid()).toBe('preferred-id');
    expect(randomUUID).toHaveBeenCalledOnce();
    vi.unstubAllGlobals();
  });

  it('creates an RFC 4122 version 4 identifier on HTTP-safe fallback', () => {
    vi.stubGlobal('crypto', undefined);
    expect(uuid()).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
    vi.unstubAllGlobals();
  });
});
