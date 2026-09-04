import type { Fragment } from './fragment';
/**
 * Build a JSDoc-style docblock fragment from an array of lines.
 *
 * Empty or `undefined` input returns `undefined` so the helper composes
 * naturally with the {@link fragment} tag's optional-interpolation
 * behavior — a node's `docs` attribute can be threaded straight in
 * without a ternary guard:
 *
 * ```ts
 * fragment`${getDocblockFragment(node.docs)}\nexport interface X {}`;
 * ```
 *
 * Single-line input renders as a one-line block (`/** line *\/`);
 * multi-line input renders as a standard multi-line JSDoc block. Empty
 * elements in the array render as bare ` *` lines, useful for paragraph
 * breaks inside a docblock.
 *
 * The helper defangs any literal `*\/` sequences inside the lines (they are
 * rewritten as `*\\/`) so that user-supplied content cannot accidentally
 * close the docblock early.
 *
 * @param lines - The lines of the docblock, or `undefined`. Empty array
 * and `undefined` both return `undefined`.
 * @param options - Optional settings.
 * @param options.withLineJump - When `true`, appends a trailing `\n` after
 * the closing `*\/`. Useful when the docblock is followed by a same-line
 * item like an enum variant.
 * @return A {@link Fragment} carrying the rendered docblock, or `undefined`
 * when `lines` is empty or `undefined`.
 *
 * @example
 * ```ts
 * import { getDocblockFragment } from '@codama/fragments/javascript';
 *
 * getDocblockFragment(['Greets the user.'])?.content;
 * // /** Greets the user. *\/
 *
 * getDocblockFragment(['First line.', '', 'Second paragraph.'])?.content;
 * // /**
 * //  * First line.
 * //  *
 * //  * Second paragraph.
 * //  *\/
 *
 * getDocblockFragment(undefined);
 * // undefined
 * ```
 */
export declare function getDocblockFragment(lines: readonly string[] | undefined, options?: {
    withLineJump?: boolean;
}): Fragment | undefined;
//# sourceMappingURL=getDocblockFragment.d.ts.map