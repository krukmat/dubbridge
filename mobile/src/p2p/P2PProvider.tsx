import {
  createContext,
  type ReactNode,
  useContext,
  useRef,
  useSyncExternalStore,
} from "react";

import { P2PService, type P2PRuntimeSnapshot } from "./P2PService";

const P2PServiceContext = createContext<P2PService | undefined>(undefined);

export function P2PProvider({ children }: { children: ReactNode }) {
  const serviceRef = useRef<P2PService | null>(null);
  if (serviceRef.current === null) serviceRef.current = new P2PService();

  return (
    <P2PServiceContext.Provider value={serviceRef.current}>
      {children}
    </P2PServiceContext.Provider>
  );
}

export function useP2PService(): P2PService {
  const service = useContext(P2PServiceContext);
  if (!service) throw new Error("useP2PService must be used within a P2PProvider");
  return service;
}

/** Selective external-store boundary for the few consumers that need runtime status. */
export function useP2PRuntimeSnapshot(): P2PRuntimeSnapshot {
  const service = useP2PService();
  return useSyncExternalStore(service.subscribe, service.getSnapshot, service.getSnapshot);
}
