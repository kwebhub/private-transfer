import type { ImportInfo, ImportMap } from './ImportMap';
/**
 * Merge multiple import maps into one. Modules and identifiers from later
 * maps are layered over earlier ones; collisions on the same `usedIdentifier`
 * are resolved by {@link preferIncoming}.
 *
 * The merge is a pure function: input maps are not mutated. The returned map
 * is frozen.
 *
 * @param importMaps - The import maps to merge, in priority order.
 * @return A frozen import map that contains every entry from every input.
 *
 * @example
 * ```ts
 * import { addToImportMap, createImportMap, mergeImportMaps } from '@codama/fragments/javascript';
 *
 * const a = addToImportMap(createImportMap(), './foo', ['Foo']);
 * const b = addToImportMap(createImportMap(), './bar', ['Bar']);
 * const merged = mergeImportMaps([a, b]);
 * ```
 */
export declare function mergeImportMaps(importMaps: readonly ImportMap[]): ImportMap;
/**
 * Decide whether an incoming `ImportInfo` should replace an existing entry
 * for the same `usedIdentifier`.
 *
 * The single rule we apply: if both refer to the same source identifier and
 * one is type-only while the other is a value import, the value import wins.
 * In every other "tied" case the existing entry stays. This keeps a value
 * import from being silently downgraded to type-only when both are
 * encountered.
 */
export declare function preferIncoming(existing: ImportInfo | undefined, incoming: ImportInfo): boolean;
//# sourceMappingURL=mergeImportMaps.d.ts.map