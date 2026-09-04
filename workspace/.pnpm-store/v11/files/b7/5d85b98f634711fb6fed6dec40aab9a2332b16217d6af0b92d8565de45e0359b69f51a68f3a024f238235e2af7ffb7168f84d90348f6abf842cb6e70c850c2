import { type Node, type NodeKind } from '@codama/nodes';
import { type Visitor } from '../visitor';
/**
 * Merge visitor: traverses the tree collecting per-node values into
 * a single result via a user-supplied `merge` function. Leaf nodes
 * (or nodes outside `keys`) yield `leafValue(node)`; every other
 * visited node's value is `merge(node, [<visited children's values>])`.
 */
export declare function mergeVisitor<TReturn, TNodeKind extends NodeKind = NodeKind>(leafValue: (node: Node) => TReturn, merge: (node: Node, values: TReturn[]) => TReturn, options?: {
    keys?: TNodeKind[];
}): Visitor<TReturn, TNodeKind>;
//# sourceMappingURL=mergeVisitor.d.ts.map