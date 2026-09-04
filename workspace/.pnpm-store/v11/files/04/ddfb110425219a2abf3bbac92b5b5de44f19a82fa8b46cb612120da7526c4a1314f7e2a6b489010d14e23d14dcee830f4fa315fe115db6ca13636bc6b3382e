import type { TypeNode } from './TypeNode';
/** Wraps another type and asserts a fixed total byte size. Padding or truncation is applied as needed. */
export interface FixedSizeTypeNode<TType extends TypeNode = TypeNode> {
    readonly kind: 'fixedSizeTypeNode';
    /** The total byte size the wrapped type must occupy. */
    readonly size: number;
    /** The wrapped type whose serialisation is constrained. */
    readonly type: TType;
}
//# sourceMappingURL=FixedSizeTypeNode.d.ts.map