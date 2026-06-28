import { StyleSheet, Text, TextInput, View } from 'react-native';

import { ActionBar, ACTION_BAR_CONTENT_HEIGHT } from '../components/ActionBar';
import { Button } from '../components/Button';
import { Panel } from '../components/Panel';
import { Screen } from '../components/Screen';
import { ScreenHeader } from '../components/ScreenHeader';
import { SelectField } from '../components/Select';
import { StateView } from '../components/StateView';
import { color, fieldStyle, space, type } from '../theme';
import { type RightsFormData, type ValidationErrors, type UploadViewState, useUploadFlow } from './useUploadFlow';

const LICENSE_TYPE_OPTIONS = [
  { label: 'Exclusive', value: 'exclusive' },
  { label: 'Non-exclusive', value: 'non_exclusive' },
  { label: 'Creative Commons', value: 'creative_commons' },
];

const SOURCE_TYPE_OPTIONS = [
  { label: 'Original', value: 'original' },
  { label: 'Licensed', value: 'licensed' },
  { label: 'Public domain', value: 'public_domain' },
  { label: 'Direct upload', value: 'direct_upload' },
];

const STEP_LABELS = ['Rights', 'File', 'Finalize'] as const;

function stepIndexFor(kind: UploadViewState['kind']): number {
  if (kind === 'rights_form') return 0;
  if (kind === 'file_pending') return 1;
  return 2;
}

const stepStyles = StyleSheet.create({
  row: { flexDirection: 'row', alignItems: 'center', marginBottom: space.xl },
  item: { flex: 1, alignItems: 'center', position: 'relative' },
  dot: { width: 28, height: 28, borderRadius: 14, alignItems: 'center', justifyContent: 'center', marginBottom: space.xs },
  dotActive: { backgroundColor: color.primary },
  dotInactive: { backgroundColor: color.raised, borderWidth: 1.5, borderColor: color.border },
  dotNum: { ...type.label },
  dotNumActive: { color: color.onPrimary },
  dotNumInactive: { color: color.ink300 },
  dotCheck: { ...type.label, color: color.onPrimary },
  label: { ...type.label },
  labelActive: { color: color.primary },
  labelInactive: { color: color.ink300 },
  connector: { position: 'absolute', top: 14, left: '50%', right: '-50%', height: 1.5, backgroundColor: color.border, zIndex: -1 },
});

function StepProgress({ steps, currentIndex, testID }: { steps: readonly string[]; currentIndex: number; testID?: string }) {
  return (
    <View testID={testID} style={stepStyles.row} accessibilityRole="progressbar">
      {steps.map((label, i) => {
        const active = i === currentIndex;
        const done = i < currentIndex;
        return (
          <View key={label} style={stepStyles.item}>
            <View style={[stepStyles.dot, active || done ? stepStyles.dotActive : stepStyles.dotInactive]}>
              {done ? <Text style={stepStyles.dotCheck}>✓</Text> : <Text style={[stepStyles.dotNum, active ? stepStyles.dotNumActive : stepStyles.dotNumInactive]}>{i + 1}</Text>}
            </View>
            <Text style={[stepStyles.label, active ? stepStyles.labelActive : stepStyles.labelInactive]}>{label}</Text>
            {i < steps.length - 1 ? <View style={stepStyles.connector} /> : null}
          </View>
        );
      })}
    </View>
  );
}

function RightsFormBody({ fields, errors, onFieldChange }: { fields: RightsFormData; errors: ValidationErrors; onFieldChange: (field: keyof RightsFormData, value: string) => void }) {
  return (
    <View style={styles.form}>
      <TextInput testID="upload-field-owner" style={[fieldStyle, errors.owner ? styles.fieldError : undefined]} placeholder="Owner" value={fields.owner} onChangeText={(v) => onFieldChange('owner', v)} />
      {errors.owner ? <Text testID="upload-error-owner" style={styles.errorText}>{errors.owner}</Text> : null}
      <SelectField label="License type" testID="upload-field-license-type" options={LICENSE_TYPE_OPTIONS} value={fields.license_type} onChange={(v) => onFieldChange('license_type', v)} error={errors.license_type} errorTestID="upload-error-license-type" />
      <SelectField label="Source type" testID="upload-field-source-type" options={SOURCE_TYPE_OPTIONS} value={fields.source_type} onChange={(v) => onFieldChange('source_type', v)} error={errors.source_type} errorTestID="upload-error-source-type" />
      <TextInput testID="upload-field-proof-reference" style={[fieldStyle, errors.proof_reference ? styles.fieldError : undefined]} placeholder="Proof reference" value={fields.proof_reference} onChangeText={(v) => onFieldChange('proof_reference', v)} />
      {errors.proof_reference ? <Text testID="upload-error-proof-reference" style={styles.errorText}>{errors.proof_reference}</Text> : null}
    </View>
  );
}

function UploadBody({ viewState, validationErrors, onFieldChange, onPickFile, onSetViewState }: { viewState: UploadViewState; validationErrors: ValidationErrors; onFieldChange: (f: keyof RightsFormData, v: string) => void; onPickFile: (rights: RightsFormData) => Promise<void>; onSetViewState: (s: UploadViewState) => void }) {
  if (viewState.kind === 'rights_form') return <RightsFormBody fields={viewState.fields} errors={validationErrors} onFieldChange={onFieldChange} />;
  if (viewState.kind === 'file_pending') return <Panel><Button testID="upload-pick-file" label="Pick file" variant="secondary" onPress={() => void onPickFile(viewState.rights)} /></Panel>;
  if (viewState.kind === 'ready') return <Panel><Text style={styles.fileName} numberOfLines={1}>{viewState.file.name}</Text></Panel>;
  if (viewState.kind === 'processing') return <StateView kind="loading" title="Uploading…" />;
  return <Panel><Text style={styles.errorText}>{viewState.message}</Text><Button label="Try again" variant="secondary" onPress={() => onSetViewState(viewState.recovery)} /></Panel>;
}

export function UploadScreen({ gatewayBaseUrl, onSuccess }: { gatewayBaseUrl: string; onSuccess: () => void }) {
  const { viewState, setViewState, validationErrors, handleFieldChange, handleRightsSubmit, handlePickFile, handleFinalize } = useUploadFlow(gatewayBaseUrl, onSuccess);
  const actionBarHeight = ACTION_BAR_CONTENT_HEIGHT + space.md * 2;
  const stepIndex = stepIndexFor(viewState.kind);
  const showProgress = viewState.kind !== 'processing' && viewState.kind !== 'error';

  return (
    <View style={styles.container}>
      <Screen testID="upload-screen" edges={["bottom"]} extraBottomPadding={actionBarHeight}>
        <ScreenHeader kicker="Upload" title="New asset" />
        {showProgress ? <StepProgress steps={STEP_LABELS} currentIndex={stepIndex} testID="upload-step-progress" /> : null}
        <UploadBody viewState={viewState} validationErrors={validationErrors} onFieldChange={handleFieldChange} onPickFile={handlePickFile} onSetViewState={setViewState} />
      </Screen>
      {viewState.kind === 'rights_form' ? <ActionBar><Button testID="upload-submit-rights" label="Continue" onPress={handleRightsSubmit} fullWidth /></ActionBar> : null}
      {viewState.kind === 'ready' ? <ActionBar><Button testID="upload-finalize" label="Upload & finalize" onPress={() => void handleFinalize(viewState.rights, viewState.file)} fullWidth /></ActionBar> : null}
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: color.canvas },
  form: { gap: space.md },
  fieldError: { borderColor: color.danger },
  fileName: { ...type.meta, color: color.ink700 },
  errorText: { ...type.meta, color: color.danger },
});
