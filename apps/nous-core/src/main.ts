import { parseArgs } from "node:util";
import { randomBytes } from "node:crypto";
import { loadConfig } from "./config.js";
import { claimInstance } from "./discovery.js";
import { startKernel } from "./process.js";
import { createCore } from "./server.js";
import { ModelRuntime } from "./model/runtime.js";

async function main(){
  const {values}=parseArgs({options:{config:{type:"string",default:"nous.toml"}}});
  const config=await loadConfig(values.config!);
  const instance=await claimInstance(config.dataRoot);
  let kernel:Awaited<ReturnType<typeof startKernel>>|undefined;
  let app:Awaited<ReturnType<typeof createCore>>|undefined;
  try{
    kernel=await startKernel(config.kernelExecutable,config.kernelConfig);
    const token=randomBytes(32).toString("hex");
    app=await createCore({kernel:kernel.client,token,consumers:config.consumers,maxUploadBytes:config.maxUploadBytes,models:ModelRuntime.withRoles(config.models)});
    const endpoint=await app.listen({host:"127.0.0.1",port:config.port});
    await instance.publish(endpoint,token);
    console.log(JSON.stringify({endpoint,discovery:instance.path}));
    await new Promise<void>(resolve=>{process.once("SIGINT",resolve);process.once("SIGTERM",resolve);});
  }finally{await app?.close();await kernel?.stop();await instance.release();}
}
main().catch(error=>{console.error(error instanceof Error?error.message:String(error));process.exitCode=1;});
