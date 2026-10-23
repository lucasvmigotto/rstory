// ESLint flat config (ESLint 9). GNOME Shell extension rules that matter for review.
const gjsGlobals = {
  console: 'readonly',
  log: 'readonly',
  logError: 'readonly',
  print: 'readonly',
  printerr: 'readonly',
  imports: 'readonly',
  ARGV: 'readonly',
  TextEncoder: 'readonly',
  TextDecoder: 'readonly',
  setTimeout: 'off',
};

export default [
  { ignores: ['node_modules/**'] },
  {
    files: ['**/*.js'],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'module',
      globals: gjsGlobals,
    },
    rules: {
      'no-undef': 'error',
      'no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
      'prefer-const': 'error',
      eqeqeq: 'error',
      // GNOME review: extensions must not spawn processes or run shell commands.
      'no-restricted-syntax': [
        'error',
        {
          selector: "MemberExpression[property.name=/^(spawn|spawn_async|spawn_sync|spawn_command_line_async|spawn_command_line_sync)$/]",
          message: 'rstory extension must never spawn processes; use the daemon (open_picker).',
        },
      ],
    },
  },
];
