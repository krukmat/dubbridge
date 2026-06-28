import React, { useState } from 'react';
import * as DocumentPicker from 'expo-document-picker';
import * as FileSystem from 'expo-file-system/legacy';
import Constants from 'expo-constants';

import { createGatewayClient } from '../api/client';
import type { GatewayErrorKind, MultipartUpload } from '../api/client';
import { useAuth } from '../auth/AuthProvider';
import type { AssetSummary } from './AssetListScreen';

export type RightsFormData = {
  owner: string;
  license_type: string;
  source_type: string;
  proof_reference: string;
};

export type ValidationErrors = Partial<Record<keyof RightsFormData, string>>;

type FileAsset = { uri: string; name: string; mimeType: string };

type NonErrorState =
  | { kind: 'rights_form'; fields: RightsFormData }
  | { kind: 'file_pending'; rights: RightsFormData }
  | { kind: 'ready'; rights: RightsFormData; file: FileAsset }
  | { kind: 'processing' };

export type UploadViewState = NonErrorState | { kind: 'error'; message: string; recovery: NonErrorState };

type IngestCreateResponse = { ingest_token: string };

export const EMPTY_RIGHTS: RightsFormData = { owner: '', license_type: '', source_type: '', proof_reference: '' };

const E2E_RIGHTS: RightsFormData = { owner: 'DubBridge Studios', license_type: 'exclusive', source_type: 'original', proof_reference: 'contract-123' };

export function isE2EEnabled(): boolean {
  if (process.env.EXPO_PUBLIC_E2E_ENABLED === 'true') return true;
  const extra = Constants.expoConfig?.extra ?? (Constants.manifest as { extra?: { e2eEnabled?: unknown } } | null)?.extra;
  return extra?.e2eEnabled === true || extra?.e2eEnabled === 'true';
}

export function validateRightsFields(fields: RightsFormData): ValidationErrors {
  const errors: ValidationErrors = {};
  if (!fields.owner.trim()) errors.owner = 'Owner is required';
  if (!fields.license_type) errors.license_type = 'Select a license type';
  if (!fields.source_type) errors.source_type = 'Select a source type';
  if (!fields.proof_reference.trim()) errors.proof_reference = 'Proof reference is required';
  return errors;
}

export function httpErrorMessage(error: GatewayErrorKind): string {
  if (error.kind === 'network') return error.message;
  if (error.kind === 'forbidden') return 'You do not have permission to perform this action.';
  if (error.kind === 'http') {
    if (error.status === 413) return 'File too large. Please choose a smaller file.';
    return `Request failed (${error.status}).`;
  }
  return 'An unexpected error occurred.';
}

type GatewayClient = ReturnType<typeof createGatewayClient>;
type Auth = ReturnType<typeof useAuth>;

type IngestOutcome = { token: string } | { logout: true } | { error: string; recovery: NonErrorState };
type RightsOutcome = { ok: true } | { logout: true } | { error: string; recovery: NonErrorState };
type FinalizeOutcome = { ok: true } | { logout: true } | { error: string; recovery: NonErrorState };

async function runIngest(client: GatewayClient, auth: Auth, file: FileAsset, rights: RightsFormData): Promise<IngestOutcome> {
  const upload: MultipartUpload = { fileUri: file.uri, fileName: file.name, mimeType: file.mimeType };
  const result = await client.postMultipart<IngestCreateResponse>('/api/ingest', auth.sessionRef, upload);
  if (!result.ok) {
    if (result.error.kind === 'session_expired') { await auth.logout(); return { logout: true }; }
    return { error: httpErrorMessage(result.error), recovery: { kind: 'ready', rights, file } };
  }
  await auth.onSessionRotation(result.value.sessionRotation);
  return { token: result.value.data.ingest_token };
}

async function runRights(client: GatewayClient, auth: Auth, token: string, rights: RightsFormData, file: FileAsset): Promise<RightsOutcome> {
  const result = await client.post<Record<string, never>>(`/api/ingest/${token}/rights`, auth.sessionRef, rights);
  if (!result.ok) {
    if (result.error.kind === 'session_expired') { await auth.logout(); return { logout: true }; }
    const expired = result.error.kind === 'http' && result.error.status === 410;
    return { error: expired ? 'Ingest session expired. Please start over.' : httpErrorMessage(result.error), recovery: expired ? { kind: 'rights_form', fields: EMPTY_RIGHTS } : { kind: 'ready', rights, file } };
  }
  await auth.onSessionRotation(result.value.sessionRotation);
  return { ok: true };
}

async function runFinalize(client: GatewayClient, auth: Auth, token: string, rights: RightsFormData, file: FileAsset): Promise<FinalizeOutcome> {
  const result = await client.post<AssetSummary>(`/api/ingest/${token}/finalize`, auth.sessionRef, {});
  if (!result.ok) {
    if (result.error.kind === 'session_expired') { await auth.logout(); return { logout: true }; }
    const expired = result.error.kind === 'http' && result.error.status === 410;
    const blocked = result.error.kind === 'http' && result.error.status === 422;
    const msg = expired ? 'Ingest session expired. Please start over.' : blocked ? 'Rights are required before finalizing. Please re-enter rights details.' : httpErrorMessage(result.error);
    return { error: msg, recovery: expired || blocked ? { kind: 'rights_form', fields: EMPTY_RIGHTS } : { kind: 'ready', rights, file } };
  }
  await auth.onSessionRotation(result.value.sessionRotation);
  return { ok: true };
}

type SetState = React.Dispatch<React.SetStateAction<UploadViewState>>;

async function pickFile(rights: RightsFormData, setViewState: SetState): Promise<void> {
  if (isE2EEnabled()) {
    const uri = `${FileSystem.cacheDirectory ?? ''}dubbridge-e2e-upload.mov`;
    await FileSystem.writeAsStringAsync(uri, 'dubbridge-e2e-placeholder');
    setViewState({ kind: 'ready', rights, file: { uri, name: 'dubbridge-e2e-upload.mov', mimeType: 'video/quicktime' } });
    return;
  }
  const picked = await DocumentPicker.getDocumentAsync({ copyToCacheDirectory: true });
  if (picked.canceled) return;
  const asset = picked.assets[0];
  if (!asset) return;
  setViewState({ kind: 'ready', rights, file: { uri: asset.uri, name: asset.name ?? 'file', mimeType: asset.mimeType ?? 'application/octet-stream' } });
}

async function finalize(auth: Auth, gatewayBaseUrl: string, rights: RightsFormData, file: FileAsset, setViewState: SetState, onSuccess: () => void): Promise<void> {
  setViewState({ kind: 'processing' });
  const client = createGatewayClient({ gatewayBaseUrl });
  const ingest = await runIngest(client, auth, file, rights);
  if ('logout' in ingest) return;
  if ('error' in ingest) { setViewState({ kind: 'error', message: ingest.error, recovery: ingest.recovery }); return; }
  const rightsResult = await runRights(client, auth, ingest.token, rights, file);
  if ('logout' in rightsResult) return;
  if ('error' in rightsResult) { setViewState({ kind: 'error', message: rightsResult.error, recovery: rightsResult.recovery }); return; }
  const finalizeResult = await runFinalize(client, auth, ingest.token, rights, file);
  if ('logout' in finalizeResult) return;
  if ('error' in finalizeResult) { setViewState({ kind: 'error', message: finalizeResult.error, recovery: finalizeResult.recovery }); return; }
  onSuccess();
}

export function useUploadFlow(gatewayBaseUrl: string, onSuccess: () => void) {
  const auth = useAuth();
  const e2eEnabled = isE2EEnabled();
  const [viewState, setViewState] = useState<UploadViewState>(
    e2eEnabled ? { kind: 'file_pending', rights: E2E_RIGHTS } : { kind: 'rights_form', fields: EMPTY_RIGHTS },
  );
  const [validationErrors, setValidationErrors] = useState<ValidationErrors>({});

  function handleFieldChange(field: keyof RightsFormData, value: string) {
    setValidationErrors((prev) => ({ ...prev, [field]: undefined }));
    setViewState((prev) => {
      if (prev.kind !== 'rights_form') return prev;
      return { kind: 'rights_form', fields: { ...prev.fields, [field]: value } };
    });
  }

  function handleRightsSubmit() {
    let pendingErrors: ValidationErrors | null = null;
    setViewState((current) => {
      if (current.kind !== 'rights_form') return current;
      const errors = validateRightsFields(current.fields);
      if (Object.keys(errors).length > 0) { pendingErrors = errors; return current; }
      pendingErrors = {};
      return { kind: 'file_pending', rights: current.fields };
    });
    if (pendingErrors !== null) setValidationErrors(pendingErrors);
  }

  const handlePickFile = (rights: RightsFormData) => pickFile(rights, setViewState);
  const handleFinalize = (rights: RightsFormData, file: FileAsset) => finalize(auth, gatewayBaseUrl, rights, file, setViewState, onSuccess);

  return { viewState, setViewState, validationErrors, handleFieldChange, handleRightsSubmit, handlePickFile, handleFinalize };
}
