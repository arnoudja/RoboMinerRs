import js from "@eslint/js";
import globals from "globals";

/**
 * Lint the multi-file IIFE static scripts.
 *
 * `no-undef` stays off: scripts share globals across <script> tags without
 * modules. Prefer `eqeqeq` / `no-debugger` / eval-family rules as CI gates.
 * `no-var` is off so existing IIFEs are not a mass rewrite.
 */
export default [
  {
    ignores: [
      "node_modules/**",
      "static/js/**/tests/**",
      "static/js/common/tests/**",
    ],
  },
  {
    files: ["static/js/**/*.js"],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: "script",
      globals: {
        ...globals.browser,
      },
    },
    rules: {
      ...js.configs.recommended.rules,
      "no-undef": "off",
      "no-unused-vars": "off",
      "no-var": "off",
      "prefer-const": "off",
      eqeqeq: ["error", "always", { null: "ignore" }],
      "no-debugger": "error",
      "no-eval": "error",
      "no-implied-eval": "error",
      "no-new-func": "error",
      "no-with": "error",
      "no-empty": ["error", { allowEmptyCatch: true }],
    },
  },
];
