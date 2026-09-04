import js from "@eslint/js";
import globals from "globals";

/**
 * Lint the multi-file IIFE static scripts.
 *
 * `no-undef` / `no-unused-vars` stay off: scripts share globals across <script>
 * tags without modules. `prefer-const` is on; `no-var` is enforced for
 * `static/js/common/**` and `static/js/mining_queue/view.js`, and stays off
 * elsewhere so remaining IIFEs are not a mass rewrite.
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
      "prefer-const": "error",
      eqeqeq: ["error", "always", { null: "ignore" }],
      "no-debugger": "error",
      "no-eval": "error",
      "no-implied-eval": "error",
      "no-new-func": "error",
      "no-with": "error",
      "no-empty": ["error", { allowEmptyCatch: true }],
    },
  },
  {
    files: ["static/js/common/**/*.js"],
    rules: {
      "no-var": "error",
    },
  },
  {
    files: ["static/js/mining_queue/view.js"],
    rules: {
      "no-var": "error",
    },
  },
];
