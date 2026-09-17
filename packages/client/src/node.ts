import { createConnectTransport } from "@connectrpc/connect-node";
import { createNousClient } from "./index.js";
import { readFile } from "node:fs/promises";
import { join } from "node:path";

export function connectNous(baseUrl: string, token: string) {
  return createNousClient(createConnectTransport({ baseUrl, httpVersion: "1.1", defaultTimeoutMs: 30_000,
    interceptors: [next => async request => { request.header.set("authorization", `Bearer ${token}`); return next(request); }],
  }));
}
export async function connectNousInstance(dataRoot:string){
  const value=JSON.parse(await readFile(join(dataRoot,"runtime","core.json"),"utf8")) as {endpoint?:unknown;token?:unknown};
  if(typeof value.endpoint!=="string"||typeof value.token!=="string")throw new Error("Invalid Core discovery record");
  const url=new URL(value.endpoint);if(url.hostname!=="127.0.0.1"||url.protocol!=="http:")throw new Error("Local discovery must use loopback HTTP");
  return connectNous(value.endpoint,value.token);
}
