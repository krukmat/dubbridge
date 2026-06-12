import { render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { SessionProvider } from './SessionProvider';
import { useSession } from './SessionProvider';

function AuthenticatedShell() {
  const session = useSession();
  if (session.status !== 'authenticated') return null;
  return <p data-testid="authenticated-shell">authenticated</p>;
}

function LoadingProbe() {
  const session = useSession();
  return <span data-testid="status">{session.status}</span>;
}

beforeEach(() => {
  vi.restoreAllMocks();
  // Reset location.assign mock before each test.
  Object.defineProperty(window, 'location', {
    writable: true,
    value: { assign: vi.fn() },
  });
});

describe('SessionProvider — HP-1: session exists', () => {
  it('renders children when /api/me returns 200', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({ subject_id: 'user-1' }), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    ));

    render(
      <SessionProvider>
        <AuthenticatedShell />
      </SessionProvider>,
    );

    await waitFor(() => {
      expect(screen.getByTestId('authenticated-shell')).toBeInTheDocument();
    });

    expect(window.location.assign).not.toHaveBeenCalled();
  });

  it('passes credentials: include in the probe request', async () => {
    const fetchSpy = vi.fn().mockResolvedValueOnce(
      new Response(JSON.stringify({}), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      }),
    );
    vi.stubGlobal('fetch', fetchSpy);

    render(<SessionProvider><span /></SessionProvider>);

    await waitFor(() => {
      expect(fetchSpy).toHaveBeenCalledWith(
        '/api/me',
        expect.objectContaining({ credentials: 'include' }),
      );
    });
  });
});

describe('SessionProvider — EC-1: no session', () => {
  it('redirects to /auth/login when /api/me returns 401', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(
      new Response(null, { status: 401 }),
    ));

    render(
      <SessionProvider>
        <AuthenticatedShell />
      </SessionProvider>,
    );

    await waitFor(() => {
      expect(window.location.assign).toHaveBeenCalledWith('/auth/login');
    });

    expect(screen.queryByTestId('authenticated-shell')).not.toBeInTheDocument();
  });

  it('does not write any token to localStorage or sessionStorage on 401', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(
      new Response(null, { status: 401 }),
    ));

    const setItemSpy = vi.spyOn(Storage.prototype, 'setItem');

    render(<SessionProvider><span /></SessionProvider>);

    await waitFor(() => {
      expect(window.location.assign).toHaveBeenCalled();
    });

    expect(setItemSpy).not.toHaveBeenCalled();
  });
});

describe('SessionProvider — EC-2: session expires mid-session', () => {
  it('redirects to /auth/login when probe returns non-401 error (fail-closed)', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValueOnce(
      new Response(null, { status: 403 }),
    ));

    render(<SessionProvider><AuthenticatedShell /></SessionProvider>);

    await waitFor(() => {
      expect(window.location.assign).toHaveBeenCalledWith('/auth/login');
    });
  });
});

describe('SessionProvider — loading state', () => {
  it('renders nothing while probe is in flight', () => {
    vi.stubGlobal('fetch', vi.fn().mockReturnValue(new Promise(() => {})));

    render(
      <SessionProvider>
        <LoadingProbe />
      </SessionProvider>,
    );

    expect(screen.queryByTestId('status')).not.toBeInTheDocument();
  });
});
