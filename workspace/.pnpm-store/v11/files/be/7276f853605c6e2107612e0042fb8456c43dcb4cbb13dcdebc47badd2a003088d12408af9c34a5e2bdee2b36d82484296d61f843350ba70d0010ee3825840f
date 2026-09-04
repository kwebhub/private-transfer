import { type Node, type NodeKind } from '@codama/nodes';
import { type Visitor } from '../visitor';
/**
 * Identity visitor: rebuilds the tree node-by-node so callers can
 * intercept individual nodes via override hooks while leaving the
 * rest untouched. Returns `null` to drop a node (and its parents
 * that required it).
 */
export declare function identityVisitor<TNodeKind extends NodeKind = NodeKind>(options?: {
    keys?: TNodeKind[];
}): Visitor<Node | null, TNodeKind>;
//# sourceMappingURL=identityVisitor.d.ts.map