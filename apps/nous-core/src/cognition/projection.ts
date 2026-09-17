import { createHash } from "node:crypto";
import { Code, ConnectError } from "@connectrpc/connect";
import type { ConsumerPolicy, Projection, Segment } from "../domain.js";
import { refKey } from "../domain.js";
import { ModelRuntime } from "../model/runtime.js";

function family(segment: Segment): "memory" | "runtime" | "resource" {
  const kind = segment.sourceRefs[0]?.kind;
  return kind === "memory" || kind === "memory_revision" ? "memory" : kind === "resource" ? "resource" : "runtime";
}
const tiers = { REQUIRED: 0, PREFERRED: 1, OPTIONAL: 2, FORBIDDEN: 3 };
export class ProjectionPlanner {
  constructor(readonly policies: ReadonlyMap<string, ConsumerPolicy>, private readonly models = new ModelRuntime()) {}
  policy(id: string) {
    const policy = this.policies.get(id);
    if (!policy) throw new ConnectError("Unknown consumer policy", Code.PermissionDenied);
    return policy;
  }
  async build(batch: Projection, requested: { maxItems: number; maxTextBytes: number }, signal?: AbortSignal): Promise<Projection> {
    const policy = this.policy(batch.consumerId);
    const items = Math.min(requested.maxItems || policy.maxItems, policy.maxItems);
    const bytes = Math.min(requested.maxTextBytes || policy.maxTextBytes, policy.maxTextBytes);
    if (items < 1 || bytes < 1) throw new ConnectError("Projection budget must be positive", Code.InvalidArgument);
    const seen = new Set<string>();
    const candidates = batch.segments.filter(segment => {
      if (policy[family(segment)] === "FORBIDDEN") return false;
      const key = segment.sourceRefs.map(refKey).join("\n");
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    }).sort((a, b) => tiers[policy[family(a)]] - tiers[policy[family(b)]]);
    const bounded = budget(candidates, items, bytes, policy.materialize);
    const required = bounded.filter(s => policy[family(s)] === "REQUIRED");
    const optional = bounded.filter(s => policy[family(s)] !== "REQUIRED");
    const refined = await this.models.refine(optional, signal);
    const segments = budget([...required, ...refined.segments], items, bytes, policy.materialize);
    const degradation = [...batch.degradation, ...refined.degradation];
    for (const f of ["memory", "runtime", "resource"] as const) {
      if (policy[f] === "REQUIRED" && !segments.some(s => family(s) === f)) degradation.push({ code: "required_contribution_missing", detail: f });
    }
    const projectionId = createHash("sha256").update(JSON.stringify([policy.revision, segments])).digest("hex");
    return { ...batch, projectionId, segments, degradation };
  }
}
function budget(segments: Segment[], items: number, maxBytes: number, materialize: boolean): Segment[] {
  const result: Segment[] = [];
  let bytes = 0;
  for (const source of segments) {
    if (result.length >= items) break;
    let text = materialize ? source.text : "";
    while (Buffer.byteLength(text) > maxBytes - bytes) {
      const encoded = Buffer.from(text).subarray(0, maxBytes - bytes);
      text = new TextDecoder("utf-8", { fatal: false }).decode(encoded).replace(/\uFFFD$/u, "");
    }
    if (source.text && materialize && !text) continue;
    bytes += Buffer.byteLength(text);
    result.push({ ...source, text });
  }
  return result;
}
