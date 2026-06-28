// Flat ESLint config — the mobile half of the production-readiness gate.
//
// Scope is deliberately narrow: it enforces the production-risk anti-patterns
// (debug logging, any-typing, type-checker suppression) at the AST level, where
// regex scanning produces false positives (e.g. a <TextInput placeholder="..." />).
// It intentionally does NOT pull in Expo's full stylistic ruleset, so the gate
// stays focused and the baseline is clean. Every rule here is an error and the
// gate runs with `--max-warnings 0`: there is no warning tier.
//
// Run via `npm run lint` or `make qa-mobile`.
const tsParser = require('@typescript-eslint/parser');
const tsPlugin = require('@typescript-eslint/eslint-plugin');

module.exports = [
  {
    ignores: [
      'dist/**',
      'coverage/**',
      'android/**',
      'artifacts/**',
      '.expo/**',
      'node_modules/**',
    ],
  },
  {
    // Production app surface only. Tests (__tests__, *.test.*) keep their freedom
    // to log and assert; the gate is about what ships to users.
    files: ['src/**/*.{ts,tsx}', 'App.tsx', 'index.ts'],
    languageOptions: {
      parser: tsParser,
      parserOptions: { ecmaFeatures: { jsx: true } },
    },
    plugins: { '@typescript-eslint': tsPlugin },
    rules: {
      // Ship no debug logging. warn/error remain as deliberate diagnostics until
      // a telemetry layer exists; log/debug are rejected outright.
      'no-console': ['error', { allow: ['warn', 'error'] }],
      'no-debugger': 'error',
      // No escape hatches around the type checker.
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/ban-ts-comment': 'error',
      // SRP/SoC gates: functions must stay small and simple.
      // CC > 10 signals mixed responsibilities; > 60 lines signals missing decomposition.
      'complexity': ['error', 10],
      'max-lines-per-function': ['error', 60],
    },
  },
];
