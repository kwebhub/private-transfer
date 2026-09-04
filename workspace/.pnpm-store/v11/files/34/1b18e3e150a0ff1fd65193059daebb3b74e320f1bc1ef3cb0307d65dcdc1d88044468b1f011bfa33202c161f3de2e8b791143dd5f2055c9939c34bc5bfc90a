import type { ImportMap } from './ImportMap';
/**
 * Compute the top-level crate names actually imported, with
 * {@link RUST_CORE_IMPORTS} excluded. Useful for syncing a renderer's
 * generated `Cargo.toml` from the imports it ends up emitting.
 *
 * Imports are first resolved against the dependency map (so symbolic
 * prefixes like `'generated::…'` are expanded before crate names are
 * extracted), then the leading `::` segment of each path is collected.
 *
 * @param importMap - The import map to inspect.
 * @param dependencies - The dependency map to apply before extracting
 * crate names. Defaults to no resolution.
 * @return A {@link Set} of external crate names.
 *
 * @example
 * ```ts
 * import { addToImportMap, createImportMap, getExternalDependencies } from '@codama/fragments/rust';
 *
 * const map = addToImportMap(createImportMap(), [
 *     'borsh::BorshSerialize',
 *     'std::collections::HashMap',
 *     'generated::accounts::A',
 * ]);
 * getExternalDependencies(map, { generated: 'crate::generated' });
 * // → Set { 'borsh' }
 * ```
 */
export declare function getExternalDependencies(importMap: ImportMap, dependencies?: Record<string, string>): Set<string>;
//# sourceMappingURL=getExternalDependencies.d.ts.map