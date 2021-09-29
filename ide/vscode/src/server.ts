// Wrapper over the WASM module.
//
// Proxies all messages between the IPC
// channel and the WASM module.
//
// And provides some utilities.

import { exit } from "process";
import { RhaiLsp } from "@rhai/lsp";
import fetch, { Headers, Request, Response } from "node-fetch";

import { performance } from "perf_hooks";

// For tracing
(global as any).performance = performance;

// For reqwest
(global as any).Headers = Headers;
(global as any).Request = Request;
(global as any).Response = Response;
(global as any).Window = Object;
(global as any).fetch = fetch;

let rhai: RhaiLsp;

process.on("message", async d => {
  if (d.method === "exit") {
    exit(0);
  }

  if (typeof rhai === "undefined") {
    rhai = await RhaiLsp.initialize({
      onMessage: msg => {
        if (process.send) {
          process.send(msg);
        }
      },
    });
  }

  rhai.message(d);
});

// These are panics from Rust.
process.on("unhandledRejection", err => {
  throw err;
});
