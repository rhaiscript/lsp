import type { Readable, Writable } from "node:stream";

/**
 * Environment required for several Rhai functions.
 *
 * This is required because WebAssembly is not self-contained and is sand-boxed.
 */
export interface Environment {
  /**
   * Return the environment variable, if any.
   */
  envVar: (name: string) => string | undefined;
  /**
   * Return whether the standard error output is a tty or not.
   */
  stdErrAtty: () => boolean;
  /**
   * Read `n` bytes from the standard input.
   *
   * If the returned array is empty, EOF is reached.
   *
   * This function must not return more than `n` bytes.
   */
  stdin: Readable | ((n: bigint) => Promise<Uint8Array>);
  /**
   * Write the given bytes to the standard output returning
   * the number of bytes written.
   */
  stdout: Writable | ((bytes: Uint8Array) => Promise<number>);
  /**
   * Write the given bytes to the standard error output returning
   * the number of bytes written.
   */
  stderr: Writable | ((bytes: Uint8Array) => Promise<number>);
  /**
   * Read a file at the given path.
   */
  readFile: (path: string) => Promise<Uint8Array>;
  /**
   * Write a file at the given path.
   */
  writeFile: (path: string, data: Uint8Array) => Promise<void>;
  /**
   * Search a glob file pattern and return the matched files.
   */
  glob: (pattern: string) => Array<string>;
  /**
   * Turn an URL into a file path.
   */
  urlToFilePath: (url: string) => string;
  /**
   * Return whether a path is a directory.
   */
  isDir: (path: string) => boolean;
  /**
   * Return whether a path is absolute.
   */
  isAbsolute: (path: string) => boolean;
  /**
   * Return the path to the current working directory.
   */
  cwd: () => string;
  /**
   * Find the `Rhai.toml` config file from the given directory
   * and return the path if found.
   */
  discoverRhaiConfig: (from: string) => string | undefined;
  /**
   * Return a promise that resolves after the given time.
   * @param ms milliseconds to wait
   */
  sleep(ms: number): Promise<void>;
}

/**
 * @private
 */
export function convertEnv(env: Environment): any {
  const stdin =
    typeof env.stdin === "function" ? env.stdin : streamToReadCb(env.stdin);
  const stdout =
    typeof env.stdout === "function" ? env.stdout : streamToWriteCb(env.stdout);
  const stderr =
    typeof env.stderr === "function" ? env.stderr : streamToWriteCb(env.stderr);

  return {
    js_env_var: env.envVar,
    js_atty_stderr: env.stdErrAtty,
    js_on_stdin: stdin,
    js_on_stdout: stdout,
    js_on_stderr: stderr,
    js_glob_files: env.glob,
    js_is_absolute: env.isAbsolute,
    js_cwd: env.cwd,
    js_discover_rhai_config: env.discoverRhaiConfig,
    js_is_dir: env.isDir,
    js_url_to_file_path: env.urlToFilePath,
    js_sleep: env.sleep,
    js_read_file: env.readFile,
    js_write_file: env.writeFile,
  };
}

function streamToWriteCb(
  stream: Writable
): (bytes: Uint8Array) => Promise<number> {
  return bytes => {
    return new Promise(resolve => {
      // FIXME: we immediately resolve as it does not matter
      //   in any of the use-cases.
      stream.write(bytes);
      resolve(bytes.length);
    });
  };
}

function streamToReadCb(stream: Readable): (n: bigint) => Promise<Uint8Array> {
  // The stream EOF event callback is immediately called after the last
  // bit of data was read, however we cannot immediately signal it as we are still returning data.
  //
  // If EOF happens, subsequent stream events will not happen, not even "end" and the promise
  // will get stuck and nodejs will terminate without any errors (found it out the hard way).
  //
  // So we keep track of EOF here and immediately return 0 bytes on the next call without
  // touching the stream.
  let eof = false;

  return n => {
    // Make sure that we only resolve/reject the promise once.
    // This might not be necessary, but it's better to be safe.
    let done = false;

    return new Promise((resolve, reject) => {
      if (eof) {
        return resolve(new Uint8Array());
      }

      function onReadable() {
        const data = stream.read(Number(n));
        if (data !== null) {
          if (!done) {
            done = true;
            resolve(data);
            stream.off("readable", onReadable);
          }
        }
      }

      stream.on("readable", onReadable);

      stream.once("end", () => {
        eof = true;
        if (!done) {
          done = true;
          resolve(new Uint8Array());
        }
      });

      stream.once("error", err => {
        if (!done) {
          console.log("error");
          done = true;
          reject(err);
        }
      });
    });
  };
}
