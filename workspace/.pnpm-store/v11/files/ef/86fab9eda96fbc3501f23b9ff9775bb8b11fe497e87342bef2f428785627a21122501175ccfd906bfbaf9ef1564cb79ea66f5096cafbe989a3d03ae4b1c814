import type { AccountFieldValueNode } from '@codama/node-types';
export type AccountFieldValueNodeInput = Omit<AccountFieldValueNode, 'account' | 'kind' | 'path'> & {
    readonly account: string;
    readonly path?: string;
};
/**
 * Refers to a field of a named account's decoded data.
 * The referenced account must carry an `accountLink` so the account's layout is known.
 * Resolving the value requires reading the account state at presentation time.
 */
export declare function accountFieldValueNode(input: AccountFieldValueNodeInput): AccountFieldValueNode;
//# sourceMappingURL=AccountFieldValueNode.d.ts.map