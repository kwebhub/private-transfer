/**
 * A `RenderMap` is the in-memory data structure a code generator builds
 * up before writing anything to disk: a frozen `ReadonlyMap` keyed by
 * output path, with a `BaseFragment` (or a concrete subtype carrying
 * imports / features / …) as the value. The helpers in this module are
 * pure data operations — they construct, merge, transform, and query
 * render maps without touching the filesystem.
 *
 * {@link writeRenderMap} is the single filesystem-touching entry point
 * here: it walks a finished map and writes every entry. Renderers that
 * tie a render map to a `Visitor` (see `@codama/renderers-core`) layer
 * that on top.
 */
import type { BaseFragment } from './BaseFragment';
import { type Path } from './path';
/**
 * A frozen map keyed by output {@link Path}, with each entry holding a
 * fragment that will be written to that path. `TFragment` defaults to
 * {@link BaseFragment} but generators typically pass a richer flavor
 * (e.g. `Fragment` from `@codama/fragments/javascript`) to carry
 * imports and other per-file metadata.
 */
export type RenderMap<TFragment extends BaseFragment> = ReadonlyMap<Path, TFragment>;
export declare function createRenderMap<TFragment extends BaseFragment = BaseFragment>(): RenderMap<TFragment>;
export declare function createRenderMap<TFragment extends BaseFragment>(path: Path, content: TFragment): RenderMap<TFragment>;
export declare function createRenderMap<TFragment extends BaseFragment>(entries: Record<Path, TFragment | undefined>): RenderMap<TFragment>;
/** Add or overwrite a single `(path, fragment)` entry. */
export declare function addToRenderMap<TFragment extends BaseFragment>(renderMap: RenderMap<TFragment>, path: Path, content: TFragment): RenderMap<TFragment>;
/** Remove the entry at `path`, returning a new frozen map. */
export declare function removeFromRenderMap<TFragment extends BaseFragment>(renderMap: RenderMap<TFragment>, path: Path): RenderMap<TFragment>;
/**
 * Combine multiple render maps into one. Later maps overwrite earlier
 * entries at the same path.
 */
export declare function mergeRenderMaps<TFragment extends BaseFragment>(renderMaps: RenderMap<TFragment>[]): RenderMap<TFragment>;
/** Transform every fragment in the map, preserving the keys. */
export declare function mapRenderMapFragment<TFragment extends BaseFragment>(renderMap: RenderMap<TFragment>, fn: (fragment: TFragment, path: Path) => TFragment): RenderMap<TFragment>;
/** Async variant of {@link mapRenderMapFragment}. */
export declare function mapRenderMapFragmentAsync<TFragment extends BaseFragment>(renderMap: RenderMap<TFragment>, fn: (fragment: TFragment, path: Path) => Promise<TFragment>): Promise<RenderMap<TFragment>>;
/** Transform the `content` of every fragment in the map. */
export declare function mapRenderMapContent<TFragment extends BaseFragment>(renderMap: RenderMap<TFragment>, fn: (content: string, path: Path) => string): RenderMap<TFragment>;
/** Async variant of {@link mapRenderMapContent}. */
export declare function mapRenderMapContentAsync<TFragment extends BaseFragment>(renderMap: RenderMap<TFragment>, fn: (content: string, path: Path) => Promise<string>): Promise<RenderMap<TFragment>>;
/**
 * Look up the fragment at `path`, throwing a structured
 * {@link CodamaError} when the key is missing.
 */
export declare function getFromRenderMap<TFragment extends BaseFragment>(renderMap: RenderMap<TFragment>, path: Path): TFragment;
/**
 * Test whether the fragment at `path` contains `value`. Accepts either
 * a plain substring or a regular expression.
 */
export declare function renderMapContains<TFragment extends BaseFragment>(renderMap: RenderMap<TFragment>, path: Path, value: RegExp | string): boolean;
/**
 * Walk the render map and write every entry to disk, rooted at
 * `basePath`. Each path is joined with `basePath` via {@link joinPath}
 * and written via {@link writeFile}; the directory structure is
 * created on demand.
 */
export declare function writeRenderMap<TFragment extends BaseFragment>(renderMap: RenderMap<TFragment>, basePath: Path): void;
//# sourceMappingURL=renderMap.d.ts.map