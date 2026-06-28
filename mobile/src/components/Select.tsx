import { Pressable, ScrollView, StyleSheet, Text, View } from 'react-native';

import { color, radius, space, type } from '../theme';

export type SelectOption = {
  label: string;
  value: string;
};

export type SelectProps = {
  options: SelectOption[];
  value: string;
  onChange: (value: string) => void;
  testID?: string;
};

/**
 * Segmented selector for enum fields.
 * RN-native, token-styled, no external dependency (D10).
 */
export function Select({ options, value, onChange, testID }: SelectProps) {
  return (
    <ScrollView
      testID={testID}
      horizontal
      showsHorizontalScrollIndicator={false}
      contentContainerStyle={styles.row}
    >
      {options.map((opt) => {
        const selected = opt.value === value;
        return (
          <Pressable
            key={opt.value}
            testID={testID ? `${testID}-option-${opt.value}` : undefined}
            accessibilityRole="radio"
            accessibilityState={{ selected }}
            onPress={() => onChange(opt.value)}
            style={[styles.pill, selected ? styles.pillSelected : styles.pillUnselected]}
          >
            <Text style={[styles.label, selected ? styles.labelSelected : styles.labelUnselected]}>
              {opt.label}
            </Text>
          </Pressable>
        );
      })}
    </ScrollView>
  );
}

// Wrapper for vertical form layout (label + Select + optional error)
export type SelectFieldProps = SelectProps & {
  label: string;
  error?: string;
  errorTestID?: string;
};

export function SelectField({ label, error, errorTestID, ...selectProps }: SelectFieldProps) {
  return (
    <View style={styles.field}>
      <Text style={styles.fieldLabel}>{label}</Text>
      <Select {...selectProps} />
      {error ? (
        <Text testID={errorTestID} style={styles.errorText}>
          {error}
        </Text>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: 'row',
    gap: space.sm,
  },
  pill: {
    paddingHorizontal: space.lg,
    paddingVertical: space.sm,
    borderRadius: radius.pill,
    borderWidth: 1,
  },
  pillSelected: {
    backgroundColor: color.primary,
    borderColor: color.primary,
  },
  pillUnselected: {
    backgroundColor: color.raised,
    borderColor: color.borderStrong,
  },
  label: {
    ...type.label,
  },
  labelSelected: {
    color: color.onPrimary,
  },
  labelUnselected: {
    color: color.ink700,
  },
  field: {
    gap: space.xs,
  },
  fieldLabel: {
    ...type.label,
    color: color.ink500,
  },
  errorText: {
    ...type.meta,
    color: color.danger,
  },
});
