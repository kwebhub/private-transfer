import type { ImportMap } from './ImportMap';
/**
 * Render an import map as a block of TypeScript `import { … } from '…';`
 * statements.
 *
 * The map is first resolved against `dependencies` so symbolic module
 * names (e.g. `'solanaAddresses'`, `'generatedAccounts'`) expand to real
 * specifiers; renderers that don't use symbolic names can omit the
 * argument.
 *
 * Output rules:
 *   - Modules are sorted with non-relative paths first, then relative;
 *     within each group, alphabetical.
 *   - Identifiers within each module are alphabetical.
 *   - When every import from a given module is type-only, the line is
 *     emitted as `import type { … } from '…';` rather than per-identifier
 *     `type` — matching the `@solana/eslint-config-solana`
 *     consolidate-type-imports convention used across the Codama
 *     published surface.
 *
 * @param importMap - The import map to render.
 * @param dependencies - The dependency map to apply before rendering.
 * Defaults to no resolution.
 * @return The block of import lines, or the empty string if the map is
 * empty.
 *
 * @example
 * ```ts
 * import { addToImportMap, createImportMap, importMapToString } from '@codama/fragments/javascript';
 *
 * let map = createImportMap();
 * map = addToImportMap(map, '@codama/spec', ['type Spec']);
 * map = addToImportMap(map, '../shared', ['CamelCaseString']);
 * importMapToString(map);
 * // import type { Spec } from '@codama/spec';
 * // import { CamelCaseString } from '../shared';
 * ```
 */
export declare function importMapToString(importMap: ImportMap, dependencies?: Record<string, string>): string;
//# sourceMappingURL=importMapToString.d.ts.map