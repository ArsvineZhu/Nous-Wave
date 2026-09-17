import { readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { parse } from "smol-toml";
import { z } from "zod";

const requirement=z.enum(["REQUIRED","PREFERRED","OPTIONAL","FORBIDDEN"]);
const consumer=z.object({consumer_id:z.string().min(1).max(128),revision:z.string().min(1).default("1"),memory:requirement.default("PREFERRED"),runtime:requirement.default("OPTIONAL"),resource:requirement.default("OPTIONAL"),max_items:z.number().int().min(1).max(2048).default(32),max_text_bytes:z.number().int().min(1).max(1048576).default(32768),materialize:z.boolean().default(true)}).strict();
const schema=z.object({
  kernel_executable:z.string().min(1),kernel_config:z.string().min(1),data_root:z.string().min(1),
  port:z.number().int().min(0).max(65535).default(9470),max_upload_bytes:z.number().int().positive().max(Number.MAX_SAFE_INTEGER).default(8*1024**3),
  consumers:z.array(consumer).min(1).max(64),
  models:z.object({steward:z.string().min(1).optional(),embedding:z.string().min(1).optional(),formation:z.string().min(1).optional(),interpretation:z.string().min(1).optional()}).strict().default({}),
}).strict();
export async function loadConfig(path:string){
  const base=dirname(resolve(path));
  const input=parse(await readFile(path,"utf8"));
  if(process.env.NOUS_CORE_PORT!==undefined)input.port=Number(process.env.NOUS_CORE_PORT);
  const config=schema.parse(input);
  if(new Set(config.consumers.map(c=>c.consumer_id)).size!==config.consumers.length)throw new Error("Duplicate consumer policy ID");
  return {kernelExecutable:resolve(base,config.kernel_executable),kernelConfig:resolve(base,config.kernel_config),dataRoot:resolve(base,config.data_root),port:config.port,maxUploadBytes:config.max_upload_bytes,models:config.models,
    consumers:config.consumers.map(c=>({consumerId:c.consumer_id,revision:c.revision,memory:c.memory,runtime:c.runtime,resource:c.resource,maxItems:c.max_items,maxTextBytes:c.max_text_bytes,materialize:c.materialize}))};
}
