import type { DefinedTypeNode, TypeNode } from '@codama/node-types';
import { DocsInput } from '../shared';
export type DefinedTypeNodeInput<TType extends TypeNode = TypeNode> = Omit<DefinedTypeNode<TType>, 'docs' | 'kind' | 'name'> & {
    readonly name: string;
    readonly docs?: DocsInput;
};
/** A reusable named type that can be referenced by `definedTypeLinkNode` from elsewhere in the IDL. */
export declare function definedTypeNode<const TType extends TypeNode>(input: DefinedTypeNodeInput<TType>): DefinedTypeNode<TType>;
//# sourceMappingURL=DefinedTypeNode.d.ts.map