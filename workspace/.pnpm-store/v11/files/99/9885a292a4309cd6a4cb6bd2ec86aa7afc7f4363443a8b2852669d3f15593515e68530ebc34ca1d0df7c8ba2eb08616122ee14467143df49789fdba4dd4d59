import type { PdaNode, PdaSeedNode } from '@codama/node-types';
import { DocsInput } from '../shared';
export type PdaNodeInput<TSeeds extends Array<PdaSeedNode> | undefined = Array<PdaSeedNode> | undefined> = Omit<PdaNode<TSeeds>, 'docs' | 'kind' | 'name'> & {
    readonly name: string;
    readonly docs?: DocsInput;
};
/** A program-derived address: its name, optional program ID override, and the seeds used to derive it. */
export declare function pdaNode<const TSeeds extends Array<PdaSeedNode> | undefined>(input: PdaNodeInput<TSeeds>): PdaNode<TSeeds>;
//# sourceMappingURL=PdaNode.d.ts.map