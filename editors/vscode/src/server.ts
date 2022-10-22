import fs from "fs";
import fsPromise from "fs/promises";
import path from "path";
import { exit } from "process";
import { RpcMessage, RhaiLsp } from "@rhaiscript/lsp";
import glob from "fast-glob";

let rhai: RhaiLsp;

process.on("message", async (d: RpcMessage) => {
  if (d.method === "exit") {
    exit(0);
  }

  if (typeof rhai === "undefined") {
    rhai = await RhaiLsp.initialize(
      {
        cwd: () => process.cwd(),
        envVar: name => process.env[name],
        discoverRhaiConfig: from => {
          const fileNames = ["Rhai.toml"];

          for (const name of fileNames) {
            try {
              const fullPath = path.join(from, name);
              fs.accessSync(fullPath);
              return fullPath;
            } catch {}
          }
        },
        glob: p => glob.sync(p),
        isAbsolute: p => path.isAbsolute(p),
        readFile: path => fsPromise.readFile(path),
        writeFile: (path, data) => fsPromise.writeFile(path, data),
        stderr: process.stderr,
        stdErrAtty: () => process.stderr.isTTY,
        stdin: process.stdin,
        stdout: process.stdout,
        urlToFilePath: (url: string) => {
          const c = decodeURIComponent(url).slice("file://".length);

          if (process.platform === "win32" && c.startsWith("/")) {
            return c.slice(1);
          }

          return c;
        },
        isDir: path => {
          try {
            return fs.statSync(path).isDirectory();
          } catch {
            return false;
          }
        },
        sleep: ms => new Promise<void>(resolve => setTimeout(resolve, ms)),
      },
      {
        onMessage(message) {
          process.send(message);
        },
      }
    );
  }

  rhai.send(d);
});

// These are panics from Rust.
process.on("unhandledRejection", up => {
  throw up;
});
