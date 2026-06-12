import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { gatewayClient } from '../api/gatewayClient';

type SessionState =
  | { status: 'loading' }
  | { status: 'authenticated' };

const SessionContext = createContext<SessionState>({ status: 'loading' });

export function useSession(): SessionState {
  return useContext(SessionContext);
}

type Props = { children: ReactNode };

export function SessionProvider({ children }: Props) {
  const [session, setSession] = useState<SessionState>({ status: 'loading' });

  useEffect(() => {
    gatewayClient.get<unknown>('/api/me').then((result) => {
      if (result.ok) {
        setSession({ status: 'authenticated' });
      } else if (result.error.kind === 'session_expired') {
        window.location.assign('/auth/login');
      } else {
        // forbidden or network error — redirect to login as fail-closed
        window.location.assign('/auth/login');
      }
    });
  }, []);

  // Mid-session 401: components can call this to trigger re-login.
  if (session.status === 'loading') {
    return null;
  }

  return (
    <SessionContext.Provider value={session}>
      {children}
    </SessionContext.Provider>
  );
}

