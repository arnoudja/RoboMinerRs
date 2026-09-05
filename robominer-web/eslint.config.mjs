import js from "@eslint/js";
import globals from "globals";

/**
 * Lint the multi-file IIFE static scripts.
 *
 * `no-undef` / `no-unused-vars` stay off: scripts share globals across <script>
 * tags without modules. `prefer-const` is on; `no-var` is enforced for
 * page-scoped IIFEs (`common/`, `mining_queue/**`, shop/mining_results/robot,
 * `rally_animation/**`, `edit_code/**`). Generated contract JS is ignored.
 */
export default [
  {
    ignores: [
      "node_modules/**",
      "static/js/**/tests/**",
      "static/js/common/tests/**",
      "static/js/**/generated/**",
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
    files: ["static/js/mining_queue/**/*.js"],
    rules: {
      "no-var": "error",
    },
  },
  {
    files: [
      "static/js/shop/page.js",
      "static/js/mining_results/page.js",
      "static/js/robot/page.js",
    ],
    rules: {
      "no-var": "error",
    },
  },
  {
    files: ["static/js/rally_animation/**/*.js"],
    rules: {
      "no-var": "error",
    },
  },
  {
    files: ["static/js/edit_code/**/*.js"],
    rules: {
      "no-var": "error",
    },
  },
];
