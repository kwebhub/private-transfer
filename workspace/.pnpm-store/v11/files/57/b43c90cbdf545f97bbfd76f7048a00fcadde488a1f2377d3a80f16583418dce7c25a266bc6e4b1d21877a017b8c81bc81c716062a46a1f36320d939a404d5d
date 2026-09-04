import { type Node, type NodeKind } from '@codama/nodes';
import { type Visitor } from './visitor';
/**
 * Identity visitor: rebuilds the tree node-by-node so callers can
 * intercept individual nodes via override hooks while leaving the rest
 * untouched. Returns `null` to drop a node (and its parents that
 * required it).
 *
 * The mechanical walk lives in `./generated/identityVisitor` (one
 * branch per spec node, derived from the attribute structure of
 * `@codama/spec`). This wrapper layers a handful of *semantic*
 * overrides on top — transformations that aren't derivable from the
 * spec alone:
 *
 *   - `enumStructVariantTypeNode` / `enumTupleVariantTypeNode`:
 *     downgrade to `enumEmptyVariantTypeNode` when the payload is
 *     empty (no fields / no items).
 *   - `hiddenPrefixTypeNode` / `hiddenSuffixTypeNode`: drop the
 *     wrapper when the prefix/suffix array is empty.
 *   - `conditionalValueNode`: return `null` when both `ifTrue` and
 *     `ifFalse` are absent post-walk.
 *   - `resolverValueNode`: collapse an empty `dependsOn` array back
 *     to `undefined` so equality checks remain stable.
 */
export declare function identityVisitor<TNodeKind extends NodeKind = NodeKind>(options?: {
    keys?: TNodeKind[];
}): Visitor<Node | null, TNodeKind>;
//# sourceMappingURL=identityVisitor.d.ts.map