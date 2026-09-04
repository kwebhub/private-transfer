import type { CamelCaseString } from '../../brands';
import type { ProgramLinkNode } from './ProgramLinkNode';
/** A reference to a defined type — possibly in a different program. */
export interface DefinedTypeLinkNode<TProgram extends ProgramLinkNode | undefined = ProgramLinkNode | undefined> {
    readonly kind: 'definedTypeLinkNode';
    /** The name of the referenced defined type. */
    readonly name: CamelCaseString;
    /** The program the referenced type is defined in. When omitted, the surrounding program is assumed. */
    readonly program?: TProgram;
}
//# sourceMappingURL=DefinedTypeLinkNode.d.ts.map