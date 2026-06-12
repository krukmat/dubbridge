import { useSession } from './auth/SessionProvider';

export function App() {
  const session = useSession();

  return (
    <main data-testid="app-shell">
      {session.status === 'authenticated' && (
        <p data-testid="authenticated-shell">Welcome to DubBridge</p>
      )}
    </main>
  );
}
