type PackageJson = {
    name: string;
    dependencies?: Record<string, string>;
    devDependencies?: Record<string, string>;
    scripts?: Record<string, string>;
    packageManager?: string;
    [key: string]: unknown;
};
/**
 * Reads the local `package.json` if one exists. Returns `undefined` when none is found so callers
 * can degrade gracefully (e.g. when running outside a Node project or via `npx`).
 */
export declare function tryGetPackageJson(): Promise<PackageJson | undefined>;
/** Reads the local `package.json`, throwing when none exists. */
export declare function getPackageJson(): Promise<PackageJson>;
/** Lists declared dependencies, returning `[]` outside a Node project (no `package.json`). */
export declare function getPackageJsonDependencies(options?: {
    includeDev?: boolean;
}): Promise<string[]>;
export {};
//# sourceMappingURL=packageJson.d.ts.map