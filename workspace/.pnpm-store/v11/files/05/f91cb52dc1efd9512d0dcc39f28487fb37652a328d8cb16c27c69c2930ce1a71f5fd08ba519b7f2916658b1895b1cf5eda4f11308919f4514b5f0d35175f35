import type { ConditionalValueCondition, ConditionalValueNode, InstructionInputValueNode, ValueNode } from '@codama/node-types';
export type ConditionalValueNodeInput<TCondition extends ConditionalValueCondition = ConditionalValueCondition, TValue extends ValueNode | undefined = ValueNode | undefined, TIfTrue extends InstructionInputValueNode | undefined = InstructionInputValueNode | undefined, TIfFalse extends InstructionInputValueNode | undefined = InstructionInputValueNode | undefined> = Omit<ConditionalValueNode<TCondition, TValue, TIfTrue, TIfFalse>, 'kind'>;
/**
 * A branching contextual value.
 * The condition resolves to a value at instruction time; that result selects between `ifTrue` and `ifFalse`.
 */
export declare function conditionalValueNode<const TCondition extends ConditionalValueCondition, const TValue extends ValueNode | undefined = undefined, const TIfTrue extends InstructionInputValueNode | undefined = undefined, const TIfFalse extends InstructionInputValueNode | undefined = undefined>(input: ConditionalValueNodeInput<TCondition, TValue, TIfTrue, TIfFalse>): ConditionalValueNode<TCondition, TValue, TIfTrue, TIfFalse>;
//# sourceMappingURL=ConditionalValueNode.d.ts.map