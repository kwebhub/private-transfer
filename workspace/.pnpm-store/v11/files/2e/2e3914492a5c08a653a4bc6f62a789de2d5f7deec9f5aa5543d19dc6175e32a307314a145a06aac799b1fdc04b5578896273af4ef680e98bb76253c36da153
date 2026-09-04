import type { DiscriminatorNode, EventNode, TypeNode } from '@codama/node-types';
import { DocsInput } from '../shared';
export type EventNodeInput<TData extends TypeNode = TypeNode, TDiscriminators extends Array<DiscriminatorNode> | undefined = Array<DiscriminatorNode> | undefined> = Omit<EventNode<TData, TDiscriminators>, 'docs' | 'kind' | 'name'> & {
    readonly name: string;
    readonly docs?: DocsInput;
};
/** A program event: its data shape and optional discriminators used to identify it on the wire. */
export declare function eventNode<const TData extends TypeNode, const TDiscriminators extends Array<DiscriminatorNode> | undefined = undefined>(input: EventNodeInput<TData, TDiscriminators>): EventNode<TData, TDiscriminators>;
//# sourceMappingURL=EventNode.d.ts.map