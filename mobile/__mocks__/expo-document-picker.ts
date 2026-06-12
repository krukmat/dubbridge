// Manual Jest mock for expo-document-picker.
// Tests that exercise the picker should override getDocumentAsync per-test.
export const getDocumentAsync = jest.fn().mockResolvedValue({ canceled: true, assets: [] });
