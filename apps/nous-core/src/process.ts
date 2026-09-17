import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { randomBytes } from "node:crypto";
import { createInterface } from "node:readline";
import { once } from "node:events";
import { KernelClient } from "./kernel-client.js";

export interface KernelProcess { client: KernelClient; child: ChildProcessWithoutNullStreams; stop(): Promise<void> }
export async function startKernel(executable: string, configPath: string, timeoutMs = 120_000): Promise<KernelProcess> {
  const token = randomBytes(32).toString("hex");
  const child = spawn(executable, ["--config", configPath], { stdio: "pipe", windowsHide: true });
  child.stderr.pipe(process.stderr);
  const lines = createInterface({ input: child.stdout, crlfDelay: Infinity });
  let startupTimer: ReturnType<typeof setTimeout> | undefined;
  const endpoint = new Promise<string>((resolve, reject) => {
    startupTimer = setTimeout(() => reject(new Error("Kernel startup timed out")), timeoutMs);
    child.once("error", reject);
    child.once("exit", code => reject(new Error(`Kernel exited during startup (${code})`)));
    lines.once("line", line => {
      try {
        if (Buffer.byteLength(line) > 4096) throw new Error("Kernel bootstrap output exceeds bound");
        const value: unknown = JSON.parse(line);
        if (!value || typeof value !== "object" || !("endpoint" in value) || typeof value.endpoint !== "string") throw new Error("Invalid Kernel discovery");
        const url = new URL(value.endpoint);
        if (url.protocol !== "http:" || url.hostname !== "127.0.0.1") throw new Error("Kernel endpoint must be loopback");
        resolve(value.endpoint);
      } catch (error) { reject(error); }
    });
  });
  child.stdin.on("error", () => { /* Child exit is reported by its process state and RPC transport. */ });
  child.stdin.write(`${token}\n`);
  const stop = async () => {
    if (child.exitCode !== null || child.signalCode !== null) return;
    const exited = once(child, "exit");
    child.stdin.end();
    const timer = setTimeout(() => child.kill(), 5000);
    try { await exited; } finally { clearTimeout(timer); lines.close(); }
  };
  try {
    const client = KernelClient.connect(await endpoint, token);
    const health = await client.health.check({ service: "nous.wave.kernel.v1alpha1.AuthorityService" });
    if (health.status !== 1) throw new Error("Kernel is not serving");
    return { client, child, stop };
  } catch (error) {
    await stop();
    throw error;
  } finally { clearTimeout(startupTimer); }
}
