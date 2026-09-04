import type { AccountNode, DiscriminatorNode, NestedTypeNode, PdaLinkNode, StructTypeNode } from '@codama/node-types';
import { DocsInput } from '../shared';
export type AccountNodeInput<TData extends NestedTypeNode<StructTypeNode> = NestedTypeNode<StructTypeNode>, TPda extends PdaLinkNode | undefined = PdaLinkNode | undefined, TDiscriminators extends Array<DiscriminatorNode> | undefined = Array<DiscriminatorNode> | undefined> = Omit<Partial<AccountNode<TData, TPda, TDiscriminators>>, 'docs' | 'kind' | 'name'> & {
    readonly name: string;
    readonly docs?: DocsInput;
};
/** An on-chain account: its name, data structure, optional fixed size, optional PDA, and optional discriminators. */
export declare function accountNode<const TData extends NestedTypeNode<StructTypeNode> = StructTypeNode<[]>, const TPda extends PdaLinkNode | undefined = undefined, const TDiscriminators extends Array<DiscriminatorNode> | undefined = undefined>(input: AccountNodeInput<TData, TPda, TDiscriminators>): AccountNode<TData, TPda, TDiscriminators>;
//# sourceMappingURL=AccountNode.d.ts.map