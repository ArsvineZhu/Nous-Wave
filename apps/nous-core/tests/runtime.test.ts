import { describe, expect, it } from "vitest";
import { ContextCompiler } from "../src/cognition/context.js";
import { ModelRuntime } from "../src/model/runtime.js";
import { ProjectionPlanner } from "../src/cognition/projection.js";
import type { ConsumerPolicy, Projection, Segment } from "../src/domain.js";

const segment = (id: string, text = id): Segment => ({ segmentId: id, text, semanticRole: "evidence", sourceRefs: [{ kind: "memory", value: id }], evidence: [], authority: "subject_cognition", stability: "EPOCH_STABLE" });
const projection = (...segments: Segment[]): Projection => ({ projectionId: "p", consumerId: "test", sourceRuntimeRevision: 1n, segments, degradation: [] });
const policy: ConsumerPolicy = { consumerId: "test", revision: "1", memory: "PREFERRED", resource: "FORBIDDEN", runtime: "OPTIONAL", maxItems: 10, maxTextBytes: 20, materialize: true };

describe("managed context synchronization", () => {
  it("appends only to a synchronized immutable prefix and resets on source revision", () => {
    const compiler = new ContextCompiler();
    const key = { subjectId: "s", sessionId: "session", consumerId: "test" };
    const initial = compiler.compile(key, "1", projection(segment("a")));
    expect(initial.kind).toBe("RESET");
    const append = compiler.compile(key, "1", projection(segment("a"), segment("b")), initial.cursor);
    expect(append.kind).toBe("APPEND"); expect(append.projection.segments.map(s => s.segmentId)).toEqual(["b"]);
    expect(compiler.compile(key, "1", projection(segment("a", "revised"), segment("b")), append.cursor).kind).toBe("RESET");
    expect(compiler.compile({ ...key, consumerId: "another" }, "1", projection(segment("a")), initial.cursor).kind).toBe("RESET");
    expect(new ContextCompiler().compile(key, "1", projection(segment("a")), initial.cursor).kind).toBe("RESET");
  });
});
describe("projection authority boundaries", () => {
  it("rejects invented model refs and preserves deterministic source selection", async () => {
    const model = new ModelRuntime(async () => ({ selectedIds: ["invented"] }));
    const result = await new ProjectionPlanner(new Map([["test", policy]]), model).build(projection(segment("a")), { maxItems: 5, maxTextBytes: 20 });
    expect(result.segments.map(s => s.segmentId)).toEqual(["a"]);
    expect(result.degradation[0]?.code).toBe("steward_rejected");
  });
  it("enforces bytes without splitting UTF-8 and keeps required sources", async () => {
    const required = { ...policy, memory: "REQUIRED" as const };
    const planner = new ProjectionPlanner(new Map([["test", required]]), new ModelRuntime(async () => ({ selectedIds: [] })));
    const result = await planner.build(projection(segment("a", "中文中文")), { maxItems: 1, maxTextBytes: 5 });
    expect(result.segments[0]?.text).toBe("中");
    expect(result.segments).toHaveLength(1);
  });
});
