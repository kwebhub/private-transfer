import type { ImportMap } from './ImportMap';
/**
 * Append imports to an import map, returning a new frozen map. The input
 * map is not mutated.
 *
 * Aliases are not added by this function — call
 * {@link addAliasToImportMap} separately when the import needs an `as`
 * clause.
 *
 * @param importMap - The import map to extend.
 * @param paths - The Rust paths to add. May be a single string, an array,
 * or a {@link Set}. An empty array short-circuits and returns `importMap`
 * unchanged.
 * @return A frozen import map that includes the new entries.
 *
 * @example
 * ```ts
 * import { addToImportMap, createImportMap } from '@codama/fragments/rust';
 *
 * const map = addToImportMap(createImportMap(), [
 *     'borsh::BorshDeserialize',
 *     'borsh::BorshSerialize',
 * ]);
 * ```
 */
export declare function addToImportMap(importMap: ImportMap, paths: ReadonlySet<string> | string | readonly string[]): ImportMap;
//# sourceMappingURL=addToImportMap.d.ts.map