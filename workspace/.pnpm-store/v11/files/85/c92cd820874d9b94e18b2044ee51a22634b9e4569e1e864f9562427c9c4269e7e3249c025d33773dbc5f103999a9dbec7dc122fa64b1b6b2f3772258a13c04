/**
 * Renderer-specific helper layered on top of `@codama/fragments`'s
 * {@link RenderMap} data structure. The pure data operations live in
 * `@codama/fragments` so they can be shared with consumers outside the
 * renderers stack; this file adds the one piece that fragments cannot
 * pull in — the visitor wrapper, which depends on the visitor + node
 * infrastructure.
 */
import { type BaseFragment, type Path, type RenderMap } from '@codama/fragments';
import { NodeKind } from '@codama/nodes';
import { Visitor } from '@codama/visitors-core';
/**
 * Wrap a {@link Visitor} that produces a {@link RenderMap} so the
 * resulting map is written to disk under `basePath` once the visit
 * completes.
 */
export declare function writeRenderMapVisitor<TFragment extends BaseFragment = BaseFragment, TNodeKind extends NodeKind = NodeKind>(visitor: Visitor<RenderMap<TFragment>, TNodeKind>, basePath: Path): Visitor<void, TNodeKind>;
//# sourceMappingURL=renderMap.d.ts.map