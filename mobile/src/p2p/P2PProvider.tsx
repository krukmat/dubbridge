import {
  createContext,
  type ReactNode,
  useContext,
  useRef,
  useSyncExternalStore,
} from "react";

import { P2PService, type P2PRuntimeSnapshot } from "./P2PService";
import { P2PSyncController } from "./sync/P2PSyncController";

const P2PServiceContext = createContext<P2PService | undefined>(undefined);
const P2PSyncContext = createContext<P2PSyncController | undefined>(undefined);

export function P2PProvider({ children }: { children: ReactNode }) {
  const serviceRef = useRef<P2PService | null>(null);
  const syncRef = useRef<P2PSyncController | null>(null);
  if (serviceRef.current === null) serviceRef.current = new P2PService();
  if (syncRef.current === null) syncRef.current = new P2PSyncController(serviceRef.current);

  return (
    <P2PServiceContext.Provider value={serviceRef.current}>
      <P2PSyncContext.Provider value={syncRef.current}>
        {children}
      </P2PSyncContext.Provider>
    </P2PServiceContext.Provider>
  );
}

export function useP2PService(): P2PService {
  const service = useContext(P2PServiceContext);
  if (!service) throw new Error("useP2PService must be used within a P2PProvider");
  return service;
}

export function useP2PSyncController(): P2PSyncController {
  const controller = useContext(P2PSyncContext);
  if (!controller) throw new Error("useP2PSyncController must be used within a P2PProvider");
  return controller;
}

/** Selective external-store boundary for the few consumers that need runtime status. */
export function useP2PRuntimeSnapshot(): P2PRuntimeSnapshot {
  const service = useP2PService();
  return useSyncExternalStore(service.subscribe, service.getSnapshot, service.getSnapshot);
}
