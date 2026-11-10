import js from '@eslint/js';
import prettier from 'eslint-config-prettier';
import react from '@eslint-react/eslint-plugin';
import hooks from 'eslint-plugin-react-hooks';
import unusedImports from 'eslint-plugin-unused-imports';
import simpleImportSort from 'eslint-plugin-simple-import-sort';
import globals from 'globals';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  { ignores: ['dist/**', 'coverage/**', 'node_modules/**'] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    languageOptions: { globals: { ...globals.browser, ...globals.node } },
  },
  {
    files: ['src/**/*.{ts,tsx}', 'tests/**/*.{ts,tsx}'],
    plugins: {
      '@eslint-react': react,
      'react-hooks': hooks,
      'unused-imports': unusedImports,
      'simple-import-sort': simpleImportSort,
    },
    rules: {
      '@eslint-react/no-missing-key': 'error',
      '@eslint-react/no-duplicate-key': 'error',
      '@eslint-react/dom-no-unsafe-target-blank': 'error',
      '@eslint-react/dom-no-missing-button-type': 'error',
      '@eslint-react/dom-no-dangerously-set-innerhtml': 'error',
      '@eslint-react/dom-no-script-url': 'error',
      'react-hooks/rules-of-hooks': 'error',
      'react-hooks/exhaustive-deps': 'error',
      '@typescript-eslint/consistent-type-imports': 'error',
      '@typescript-eslint/no-unused-vars': 'off',
      'unused-imports/no-unused-imports': 'error',
      'unused-imports/no-unused-vars': [
        'error',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_',
          ignoreRestSiblings: true,
        },
      ],
      eqeqeq: ['error', 'smart'],
      'no-var': 'error',
      'simple-import-sort/imports': [
        'error',
        {
          groups: [
            ['^\\u0000'],
            ['^preact(?:/|$)', '^preact-router(?:/|$)'],
            ['^@?\\w'],
            ['^\\.\\.(?:/|$)'],
            ['^\\./'],
          ],
        },
      ],
      'simple-import-sort/exports': 'error',
    },
  },
  {
    files: ['src/**/*.{ts,tsx}'],
    rules: {
      'max-lines': ['error', { max: 400, skipBlankLines: true, skipComments: true }],
      'no-restricted-imports': [
        'error',
        {
          paths: [
            { name: 'react', message: 'Use preact and preact/hooks in this application.' },
            { name: 'react-dom', message: 'Use the native Preact renderer.' },
            {
              name: 'preact/compat',
              message: 'Use native Preact APIs instead of a compatibility layer.',
            },
          ],
          patterns: [
            { group: ['react/*', 'react-dom/*'], message: 'Use native Preact APIs.' },
            {
              group: ['**/lib/api'],
              message: 'Import the owning service, hook, type, or helper directly.',
            },
          ],
        },
      ],
    },
  },
  {
    files: ['tests/**/*.{ts,tsx}'],
    // Browser/service-worker fakes intentionally model partial dynamic APIs.
    rules: { '@typescript-eslint/no-explicit-any': 'off' },
  },
  {
    files: ['public/sw.js'],
    languageOptions: { globals: globals.serviceworker },
  },
  prettier,
);
