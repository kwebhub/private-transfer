import type { ImportInput, ImportMap, Module } from './ImportMap';
/**
 * Append imports to a module entry, returning a new frozen import map. The
 * input map is not mutated.
 *
 * Within a single batch, the same conflict-resolution rule that
 * {@link mergeImportMaps} uses is applied — a value import always wins over
 * a type-only import of the same identifier — so callers don't have to
 * worry about input order when passing both.
 *
 * @param importMap - The import map to extend.
 * @param module - The source module the imports come from.
 * @param imports - The shorthand strings to add. Empty array short-circuits
 * and returns `importMap` unchanged.
 * @return A frozen import map that includes the new entries.
 *
 * @example
 * ```ts
 * import { addToImportMap, createImportMap } from '@codama/fragments/javascript';
 *
 * const map = addToImportMap(createImportMap(), './foo', ['type Foo', 'Bar']);
 * ```
 */
export declare function addToImportMap(importMap: ImportMap, module: Module, imports: ImportInput[]): ImportMap;
//# sourceMappingURL=addToImportMap.d.ts.map