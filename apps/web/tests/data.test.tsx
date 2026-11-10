import { act, render, screen } from '@testing-library/preact';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { refresh, useData } from '../src/lib/api';
function Probe({ path }: { path: string }) {
  const { data, error, loading } = useData<{ name: string }>(path);
  return <div>{loading ? 'Loading' : error || data?.name}</div>;
}
afterEach(() => vi.unstubAllGlobals());
describe('query lifecycle', () => {
  it('keeps the current record visible during a background refresh', async () => {
    let resolveNext: (value: Response) => void = () => {};
    const fetcher = vi
      .fn()
      .mockResolvedValueOnce(new Response('{"name":"Current record"}'))
      .mockImplementationOnce(
        () =>
          new Promise<Response>((resolve) => {
            resolveNext = resolve;
          }),
      );
    vi.stubGlobal('fetch', fetcher);
    render(<Probe path="/entry" />);
    expect(await screen.findByText('Current record')).toBeVisible();
    act(() => refresh());
    expect(screen.queryByText('Loading')).not.toBeInTheDocument();
    expect(screen.getByText('Current record')).toBeVisible();
    await act(async () => {
      resolveNext(new Response('{"name":"Updated record"}'));
    });
    expect(await screen.findByText('Updated record')).toBeVisible();
  });
  it('does not let a stale result replace a newer query', async () => {
    let finishOld: (v: Response) => void = () => {};
    vi.stubGlobal(
      'fetch',
      vi.fn().mockImplementation((path: string) =>
        path.endsWith('/old')
          ? new Promise<Response>((resolve) => {
              finishOld = resolve;
            })
          : Promise.resolve(new Response('{"name":"Current record"}')),
      ),
    );
    const { rerender } = render(<Probe path="/old" />);
    rerender(<Probe path="/new" />);
    expect(await screen.findByText('Current record')).toBeVisible();
    await act(async () => {
      finishOld(new Response('{"name":"Stale record"}'));
    });
    expect(screen.queryByText('Stale record')).not.toBeInTheDocument();
    expect(screen.getByText('Current record')).toBeVisible();
  });
  it('refreshes real query data after a successful mutation event', async () => {
    let calls = 0;
    const fetcher = vi.fn().mockImplementation(() => {
      calls++;
      return Promise.resolve(
        new Response(JSON.stringify({ name: calls === 1 ? 'Submitted' : 'Approved' })),
      );
    });
    vi.stubGlobal('fetch', fetcher);
    render(<Probe path="/entry" />);
    expect(await screen.findByText('Submitted')).toBeVisible();
    act(() => refresh());
    expect(await screen.findByText('Approved')).toBeVisible();
  });
});
