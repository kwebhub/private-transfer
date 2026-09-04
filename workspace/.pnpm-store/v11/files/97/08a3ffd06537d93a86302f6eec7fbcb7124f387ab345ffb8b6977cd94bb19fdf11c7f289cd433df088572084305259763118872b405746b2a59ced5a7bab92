/**
 * Node-only filesystem helpers used by code generators to write their
 * output to disk. Each function checks the `__NODEJS__` build flag and
 * throws a structured {@link CodamaError} on non-Node platforms so
 * accidental calls from a browser bundle fail loudly rather than
 * silently no-oping.
 */
import { Path } from './path';
/** Create a directory (and any missing parents) at the given path. */
export declare function createDirectory(path: Path): void;
/** Recursively delete the directory at the given path, if it exists. */
export declare function deleteDirectory(path: Path): void;
/**
 * Write `content` to a file at `path`, creating intermediate
 * directories as needed.
 */
export declare function writeFile(path: Path, content: string): void;
/** Check whether a file or directory exists at the given path. */
export declare function fileExists(path: Path): boolean;
/** Read the file at the given path as a UTF-8 string. */
export declare function readFile(path: Path): string;
/**
 * Read the file at the given path as a UTF-8 string and parse it as
 * JSON. The result is typed as the caller-supplied `T`; no runtime
 * validation is performed.
 */
export declare function readJson<T>(path: Path): T;
//# sourceMappingURL=fs.d.ts.map