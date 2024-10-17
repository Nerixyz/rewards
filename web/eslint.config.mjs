import pluginVue from 'eslint-plugin-vue';
import vueTsEslintConfig from '@vue/eslint-config-typescript';

export default [
  ...pluginVue.configs['flat/essential'],
  ...vueTsEslintConfig({
    extends: ['recommended', 'eslintRecommended', 'stylistic'],
  }),
  {
    files: ['**/*.ts', '**/*.tsx', '**/*.vue', '**/*.js'],

    rules: {
      'vue/no-dupe-keys': 'off',
      'vue/multi-word-component-names': 'off',
      '@typescript-eslint/ban-ts-comment': 'off',
      'vue/max-attributes-per-line': 'off',
      'vue/singleline-html-element-content-newline': 'off',
      'vue/html-self-closing': 'off',
    },
  },
];
