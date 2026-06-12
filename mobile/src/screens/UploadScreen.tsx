import * as DocumentPicker from 'expo-document-picker';
import Constants from 'expo-constants';
import { useState } from 'react';
import {
  ActivityIndicator,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from 'react-native';

import { createGatewayClient } from '../api/client';
import type { GatewayErrorKind } from '../api/client';
import { useAuth } from '../auth/AuthProvider';
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
  if (!__DEV__) {
    return false;
  }

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
      setViewState({
        kind: 'ready',
        rights,
        file: {
          uri: 'file:///tmp/dubbridge-e2e-upload.mov',
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

    // Step 1: POST /ingest (multipart)
    const formData = new FormData();
    formData.append('title', file.name);
    formData.append('file', {
      uri: file.uri,
      name: file.name,
      type: file.mimeType,
    } as unknown as Blob);

    const ingestResult = await client.postMultipart<IngestCreateResponse>(
      '/api/ingest',
      auth.sessionRef,
      formData,
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
    <View testID="upload-screen" style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.kicker}>Upload</Text>
        <Text style={styles.title}>New asset</Text>
      </View>

      {viewState.kind === 'rights_form' ? (
        <ScrollView contentContainerStyle={styles.formContent}>
          <Text style={styles.stepLabel}>Step 1 — Rights details</Text>
          <TextInput
            testID="upload-field-owner"
            style={styles.input}
            placeholder="Owner"
            value={viewState.fields.owner}
            onChangeText={(v) => handleFieldChange('owner', v)}
          />
          <TextInput
            testID="upload-field-license-type"
            style={styles.input}
            placeholder="License type"
            value={viewState.fields.license_type}
            onChangeText={(v) => handleFieldChange('license_type', v)}
          />
          <TextInput
            testID="upload-field-source-type"
            style={styles.input}
            placeholder="Source type"
            value={viewState.fields.source_type}
            onChangeText={(v) => handleFieldChange('source_type', v)}
          />
          <TextInput
            testID="upload-field-proof-reference"
            style={styles.input}
            placeholder="Proof reference"
            value={viewState.fields.proof_reference}
            onChangeText={(v) => handleFieldChange('proof_reference', v)}
          />
          <Pressable
            testID="upload-submit-rights"
            onPress={handleRightsSubmit}
            style={styles.button}
          >
            <Text style={styles.buttonText}>Continue</Text>
          </Pressable>
        </ScrollView>
      ) : null}

      {viewState.kind === 'file_pending' ? (
        <View style={styles.panel}>
          <Text style={styles.stepLabel}>Step 2 — Pick your file</Text>
          <Pressable
            testID="upload-pick-file"
            onPress={() => void handlePickFile(viewState.rights)}
            style={styles.button}
          >
            <Text style={styles.buttonText}>Pick file</Text>
          </Pressable>
        </View>
      ) : null}

      {viewState.kind === 'ready' ? (
        <View style={styles.panel}>
          <Text style={styles.stepLabel}>Step 3 — Review and finalize</Text>
          <Text style={styles.fileName} numberOfLines={1}>
            {viewState.file.name}
          </Text>
          <Pressable
            testID="upload-finalize"
            onPress={() => void handleFinalize(viewState.rights, viewState.file)}
            style={styles.button}
          >
            <Text style={styles.buttonText}>Upload & finalize</Text>
          </Pressable>
        </View>
      ) : null}

      {viewState.kind === 'processing' ? (
        <View style={styles.panel}>
          <ActivityIndicator size="small" color="#1a5d50" />
          <Text style={styles.statusText}>Uploading…</Text>
        </View>
      ) : null}

      {viewState.kind === 'error' ? (
        <View style={styles.panel}>
          <Text style={styles.errorText}>{viewState.message}</Text>
          <Pressable
            onPress={() => setViewState(viewState.recovery)}
            style={styles.secondaryButton}
          >
            <Text style={styles.secondaryButtonText}>Try again</Text>
          </Pressable>
        </View>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f2f4ee',
    padding: 24,
    gap: 20,
  },
  header: {
    marginTop: 24,
    gap: 10,
  },
  kicker: {
    fontSize: 12,
    fontWeight: '700',
    textTransform: 'uppercase',
    color: '#537462',
  },
  title: {
    fontSize: 32,
    fontWeight: '700',
    color: '#10212a',
  },
  formContent: {
    gap: 12,
  },
  stepLabel: {
    fontSize: 13,
    fontWeight: '700',
    textTransform: 'uppercase',
    color: '#537462',
    marginBottom: 4,
  },
  input: {
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#cfdbd6',
    backgroundColor: '#ffffff',
    paddingHorizontal: 14,
    paddingVertical: 12,
    fontSize: 15,
    color: '#10212a',
  },
  panel: {
    borderRadius: 10,
    backgroundColor: '#ffffff',
    borderWidth: 1,
    borderColor: '#d7dfd7',
    padding: 20,
    gap: 12,
  },
  button: {
    alignSelf: 'flex-start',
    borderRadius: 8,
    backgroundColor: '#1a5d50',
    paddingHorizontal: 18,
    paddingVertical: 14,
  },
  buttonText: {
    fontSize: 15,
    fontWeight: '600',
    color: '#f8fbf9',
  },
  secondaryButton: {
    alignSelf: 'flex-start',
    borderRadius: 8,
    backgroundColor: '#dfe8e5',
    paddingHorizontal: 18,
    paddingVertical: 14,
  },
  secondaryButtonText: {
    fontSize: 15,
    fontWeight: '600',
    color: '#14312d',
  },
  fileName: {
    fontSize: 15,
    color: '#3c4954',
    fontFamily: 'monospace',
  },
  statusText: {
    fontSize: 15,
    color: '#3c4954',
  },
  errorText: {
    fontSize: 15,
    color: '#b91c1c',
    lineHeight: 22,
  },
});
