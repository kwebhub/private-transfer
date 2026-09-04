import type { ImportMap } from './ImportMap';
/**
 * Compute the set of external (non-relative) module specifiers an import
 * map references, with dependency-map resolution applied first. The
 * returned values are *root* package names — for `'@scope/pkg/sub'` the
 * value is `'@scope/pkg'`, and for `'pkg/sub'` it is `'pkg'`.
 *
 * Useful for syncing a renderer's generated `package.json` from the
 * imports it ends up emitting.
 *
 * @param importMap - The import map to inspect.
 * @param dependencies - The dependency map to apply before extracting
 * names. Defaults to no resolution.
 * @return A {@link Set} of external root package names.
 *
 * @example
 * ```ts
 * import { addToImportMap, createImportMap, getExternalDependencies } from '@codama/fragments/javascript';
 *
 * let map = createImportMap();
 * map = addToImportMap(map, '@solana/kit', ['Address']);
 * map = addToImportMap(map, '@solana/kit/program-client-core', ['ProgramClient']);
 * map = addToImportMap(map, '../shared', ['Local']);
 * getExternalDependencies(map);
 * // → Set { '@solana/kit' }
 * ```
 */
export declare function getExternalDependencies(importMap: ImportMap, dependencies?: Record<string, string>): Set<string>;
//# sourceMappingURL=getExternalDependencies.d.ts.map