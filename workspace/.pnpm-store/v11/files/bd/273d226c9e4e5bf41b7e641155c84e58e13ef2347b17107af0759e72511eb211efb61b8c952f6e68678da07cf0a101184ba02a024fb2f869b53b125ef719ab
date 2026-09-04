import type { StructFieldDisplayNode, StructFieldTypeNode, TypeNode, ValueNode } from '@codama/node-types';
import { DocsInput } from '../../shared';
export type StructFieldTypeNodeInput<TType extends TypeNode = TypeNode, TDefaultValue extends ValueNode | undefined = ValueNode | undefined, TDisplay extends StructFieldDisplayNode | undefined = StructFieldDisplayNode | undefined> = Omit<StructFieldTypeNode<TType, TDefaultValue, TDisplay>, 'docs' | 'kind' | 'name'> & {
    readonly name: string;
    readonly docs?: DocsInput;
};
/** A named field within a struct type. */
export declare function structFieldTypeNode<const TType extends TypeNode, const TDefaultValue extends ValueNode | undefined = undefined, const TDisplay extends StructFieldDisplayNode | undefined = undefined>(input: StructFieldTypeNodeInput<TType, TDefaultValue, TDisplay>): StructFieldTypeNode<TType, TDefaultValue, TDisplay>;
//# sourceMappingURL=StructFieldTypeNode.d.ts.map