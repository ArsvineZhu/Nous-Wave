import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { mkdir, open, readFile, unlink, writeFile } from "node:fs/promises";
import { join } from "node:path";

const execute=promisify(execFile);
export async function claimInstance(dataRoot:string){
  const runtime=join(dataRoot,"runtime");
  await mkdir(runtime,{recursive:true,mode:0o700});
  if(process.platform==="win32"){
    const {stdout}=await execute("whoami",[],{windowsHide:true});
    const user=stdout.trim();if(!user||/[\r\n]/.test(user))throw new Error("Cannot identify local credential owner");
    await execute("icacls",[runtime,"/inheritance:r","/grant:r",`${user}:(OI)(CI)F`],{windowsHide:true});
  }
  const lock=join(runtime,"instance.lock");
  try{const previous=JSON.parse(await readFile(lock,"utf8")) as {pid:number};
    if(!Number.isInteger(previous.pid)||previous.pid<=0)throw new Error("Invalid instance lock");
    let alive=true;try{process.kill(previous.pid,0);}catch(error){if((error as NodeJS.ErrnoException).code==="ESRCH")alive=false;else throw error;}
    if(alive)throw new Error("A Core instance already owns this data root");
    await unlink(lock);
  }catch(error){if((error as NodeJS.ErrnoException).code!=="ENOENT")throw error;}
  const handle=await open(lock,"wx",0o600);
  await handle.writeFile(JSON.stringify({pid:process.pid}));await handle.close();
  const path=join(runtime,"core.json");
  return {
    path,
    publish:(endpoint:string,token:string)=>writeFile(path,JSON.stringify({endpoint,token,pid:process.pid}),{mode:0o600}),
    release:async()=>{await unlink(path).catch(error=>{if(error.code!=="ENOENT")throw error;});await unlink(lock);},
  };
}
