import * as DocumentPicker from 'expo-document-picker';
import * as FileSystem from 'expo-file-system/legacy';
import Constants from 'expo-constants';
import { useState } from 'react';
import {
  StyleSheet,
  Text,
  TextInput,
  View,
} from 'react-native';

import { createGatewayClient } from '../api/client';
import type { GatewayErrorKind, MultipartUpload } from '../api/client';
import { useAuth } from '../auth/AuthProvider';
import { Button } from '../components/Button';
import { Panel } from '../components/Panel';
import { Screen } from '../components/Screen';
import { ScreenHeader } from '../components/ScreenHeader';
import { StateView } from '../components/StateView';
import { color, fieldStyle, space, type } from '../theme';
import type { AssetSummary } from './AssetListScreen';

export type RightsFormData = {
  owner: string;
  license_type: string;
  source_type: string;
  proof_reference: string;
};

type FileAsset = {
  uri: string;
  name: string;
  mimeType: string;
};

// Non-error states are the valid recovery targets.
type NonErrorState =
  | { kind: 'rights_form'; fields: RightsFormData }
  | { kind: 'file_pending'; rights: RightsFormData }
  | { kind: 'ready'; rights: RightsFormData; file: FileAsset }
  | { kind: 'processing' };

type UploadViewState =
  | NonErrorState
  | { kind: 'error'; message: string; recovery: NonErrorState };

type IngestCreateResponse = { ingest_token: string };

const EMPTY_RIGHTS: RightsFormData = {
  owner: '',
  license_type: '',
  source_type: '',
  proof_reference: '',
};

const E2E_RIGHTS: RightsFormData = {
  owner: 'DubBridge Studios',
  license_type: 'exclusive',
  source_type: 'original',
  proof_reference: 'contract-123',
};

function isE2EEnabled(): boolean {
  // EXPO_PUBLIC_E2E_ENABLED is a build-time opt-in injected only by the
  // screenshot/E2E tooling — it is never set in production or release builds.
  // It is the authoritative signal and must NOT be gated behind __DEV__: the
  // Maestro suite runs a release-mode export bundle (where __DEV__ is false)
  // because dev bundles crash when launched standalone inside the APK
  // ("Cannot create devtools websocket connections in embedded environments").
  if (process.env.EXPO_PUBLIC_E2E_ENABLED === 'true') {
    return true;
  }

  const extra =
    Constants.expoConfig?.extra ??
    (Constants.manifest as { extra?: { e2eEnabled?: unknown } } | null)?.extra;

  return extra?.e2eEnabled === true || extra?.e2eEnabled === 'true';
}

function httpErrorMessage(error: GatewayErrorKind): string {
  if (error.kind === 'network') return error.message;
  if (error.kind === 'forbidden') return 'You do not have permission to perform this action.';
  if (error.kind === 'http') {
    if (error.status === 413) return 'File too large. Please choose a smaller file.';
    return `Request failed (${error.status}).`;
  }
  return 'An unexpected error occurred.';
}

export function UploadScreen({
  gatewayBaseUrl,
  onSuccess,
}: {
  gatewayBaseUrl: string;
  onSuccess: () => void;
}) {
  const auth = useAuth();
  const e2eEnabled = isE2EEnabled();
  const [viewState, setViewState] = useState<UploadViewState>(
    e2eEnabled
      ? { kind: 'file_pending', rights: E2E_RIGHTS }
      : { kind: 'rights_form', fields: EMPTY_RIGHTS },
  );

  function handleFieldChange(field: keyof RightsFormData, value: string) {
    setViewState((prev) => {
      if (prev.kind !== 'rights_form') return prev;
      return { kind: 'rights_form', fields: { ...prev.fields, [field]: value } };
    });
  }

  function handleRightsSubmit() {
    // Functional form avoids reading stale closure state in tests.
    setViewState((current) => {
      if (current.kind !== 'rights_form') return current;
      const { fields } = current;
      if (
        !fields.owner.trim() ||
        !fields.license_type.trim() ||
        !fields.source_type.trim() ||
        !fields.proof_reference.trim()
      ) {
        return current;
      }
      return { kind: 'file_pending', rights: fields };
    });
  }

  async function handlePickFile(rights: RightsFormData) {
    if (isE2EEnabled()) {
      // Materialize a real, readable file in the app cache dir. A hardcoded
      // file:///tmp path is not readable on the device, which rejects the
      // multipart upload (FileSystem.uploadAsync -> IOException). The mock
      // gateway does not validate file bytes, so any small placeholder works.
      const uri = `${FileSystem.cacheDirectory ?? ''}dubbridge-e2e-upload.mov`;
      await FileSystem.writeAsStringAsync(uri, 'dubbridge-e2e-placeholder');
      setViewState({
        kind: 'ready',
        rights,
        file: {
          uri,
          name: 'dubbridge-e2e-upload.mov',
          mimeType: 'video/quicktime',
        },
      });
      return;
    }

    const picked = await DocumentPicker.getDocumentAsync({ copyToCacheDirectory: true });
    if (picked.canceled) return;
    const asset = picked.assets[0];
    if (!asset) return;
    setViewState({
      kind: 'ready',
      rights,
      file: {
        uri: asset.uri,
        name: asset.name ?? 'file',
        mimeType: asset.mimeType ?? 'application/octet-stream',
      },
    });
  }

  async function handleFinalize(rights: RightsFormData, file: FileAsset) {
    setViewState({ kind: 'processing' });
    const client = createGatewayClient({ gatewayBaseUrl });

    // Step 1: POST /ingest (multipart via FileSystem.uploadAsync)
    const upload: MultipartUpload = {
      fileUri: file.uri,
      fileName: file.name,
      mimeType: file.mimeType,
    };

    const ingestResult = await client.postMultipart<IngestCreateResponse>(
      '/api/ingest',
      auth.sessionRef,
      upload,
    );

    if (!ingestResult.ok) {
      if (ingestResult.error.kind === 'session_expired') {
        await auth.logout();
        return;
      }
      setViewState({
        kind: 'error',
        message: httpErrorMessage(ingestResult.error),
        recovery: { kind: 'ready', rights, file },
      });
      return;
    }
    await auth.onSessionRotation(ingestResult.value.sessionRotation);
    const ingestToken = ingestResult.value.data.ingest_token;

    // Step 2: POST /ingest/{token}/rights
    const rightsResult = await client.post<Record<string, never>>(
      `/api/ingest/${ingestToken}/rights`,
      auth.sessionRef,
      rights,
    );

    if (!rightsResult.ok) {
      if (rightsResult.error.kind === 'session_expired') {
        await auth.logout();
        return;
      }
      const expired =
        rightsResult.error.kind === 'http' && rightsResult.error.status === 410;
      setViewState({
        kind: 'error',
        message: expired
          ? 'Ingest session expired. Please start over.'
          : httpErrorMessage(rightsResult.error),
        recovery: expired
          ? { kind: 'rights_form', fields: EMPTY_RIGHTS }
          : { kind: 'ready', rights, file },
      });
      return;
    }
    await auth.onSessionRotation(rightsResult.value.sessionRotation);

    // Step 3: POST /ingest/{token}/finalize
    const finalizeResult = await client.post<AssetSummary>(
      `/api/ingest/${ingestToken}/finalize`,
      auth.sessionRef,
      {},
    );

    if (!finalizeResult.ok) {
      if (finalizeResult.error.kind === 'session_expired') {
        await auth.logout();
        return;
      }
      const expired =
        finalizeResult.error.kind === 'http' && finalizeResult.error.status === 410;
      const blocked =
        finalizeResult.error.kind === 'http' && finalizeResult.error.status === 422;
      setViewState({
        kind: 'error',
        message: expired
          ? 'Ingest session expired. Please start over.'
          : blocked
            ? 'Rights are required before finalizing. Please re-enter rights details.'
            : httpErrorMessage(finalizeResult.error),
        recovery:
          expired || blocked
            ? { kind: 'rights_form', fields: EMPTY_RIGHTS }
            : { kind: 'ready', rights, file },
      });
      return;
    }
    await auth.onSessionRotation(finalizeResult.value.sessionRotation);
    onSuccess();
  }

  return (
    <Screen testID="upload-screen" edges={["bottom"]}>
      <ScreenHeader kicker="Upload" title="New asset" />

      {viewState.kind === 'rights_form' ? (
        <View style={styles.form}>
          <Text style={styles.stepLabel}>Step 1 — Rights details</Text>
          <TextInput
            testID="upload-field-owner"
            style={fieldStyle}
            placeholder="Owner"
            value={viewState.fields.owner}
            onChangeText={(v) => handleFieldChange('owner', v)}
          />
          <TextInput
            testID="upload-field-license-type"
            style={fieldStyle}
            placeholder="License type"
            value={viewState.fields.license_type}
            onChangeText={(v) => handleFieldChange('license_type', v)}
          />
          <TextInput
            testID="upload-field-source-type"
            style={fieldStyle}
            placeholder="Source type"
            value={viewState.fields.source_type}
            onChangeText={(v) => handleFieldChange('source_type', v)}
          />
          <TextInput
            testID="upload-field-proof-reference"
            style={fieldStyle}
            placeholder="Proof reference"
            value={viewState.fields.proof_reference}
            onChangeText={(v) => handleFieldChange('proof_reference', v)}
          />
          <Button
            testID="upload-submit-rights"
            label="Continue"
            onPress={handleRightsSubmit}
          />
        </View>
      ) : null}

      {viewState.kind === 'file_pending' ? (
        <Panel>
          <Text style={styles.stepLabel}>Step 2 — Pick your file</Text>
          <Button
            testID="upload-pick-file"
            label="Pick file"
            variant="secondary"
            onPress={() => void handlePickFile(viewState.rights)}
          />
        </Panel>
      ) : null}

      {viewState.kind === 'ready' ? (
        <Panel>
          <Text style={styles.stepLabel}>Step 3 — Review and finalize</Text>
          <Text style={styles.fileName} numberOfLines={1}>
            {viewState.file.name}
          </Text>
          <Button
            testID="upload-finalize"
            label="Upload & finalize"
            onPress={() => void handleFinalize(viewState.rights, viewState.file)}
          />
        </Panel>
      ) : null}

      {viewState.kind === 'processing' ? (
        <StateView kind="loading" title="Uploading…" />
      ) : null}

      {viewState.kind === 'error' ? (
        <Panel>
          <Text style={styles.errorText}>{viewState.message}</Text>
          <Button
            label="Try again"
            variant="secondary"
            onPress={() => setViewState(viewState.recovery)}
          />
        </Panel>
      ) : null}
    </Screen>
  );
}

const styles = StyleSheet.create({
  form: { gap: space.md },
  stepLabel: { ...type.label, color: color.primary },
  fileName: { ...type.meta, color: color.ink700 },
  errorText: { ...type.body, color: color.danger },
});
