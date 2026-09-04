import type { InstructionArgumentNode, InstructionInputValueNode, StructFieldDisplayNode, TypeNode } from '@codama/node-types';
import { DocsInput } from '../shared';
export type InstructionArgumentNodeInput<TDefaultValue extends InstructionInputValueNode | undefined = InstructionInputValueNode | undefined, TType extends TypeNode = TypeNode, TDisplay extends StructFieldDisplayNode | undefined = StructFieldDisplayNode | undefined> = Omit<InstructionArgumentNode<TDefaultValue, TType, TDisplay>, 'docs' | 'kind' | 'name'> & {
    readonly name: string;
    readonly docs?: DocsInput;
};
/** A named argument of an instruction, with its type and an optional default value. */
export declare function instructionArgumentNode<const TDefaultValue extends InstructionInputValueNode | undefined = undefined, const TType extends TypeNode = TypeNode, const TDisplay extends StructFieldDisplayNode | undefined = undefined>(input: InstructionArgumentNodeInput<TDefaultValue, TType, TDisplay>): InstructionArgumentNode<TDefaultValue, TType, TDisplay>;
//# sourceMappingURL=InstructionArgumentNode.d.ts.map