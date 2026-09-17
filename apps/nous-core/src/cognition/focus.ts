import { randomUUID } from "node:crypto";
import { Code, ConnectError } from "@connectrpc/connect";
import type { Focus, FocusChange, RuntimePort } from "../domain.js";

export class FocusRuntime {
  constructor(private readonly store: RuntimePort) {}

  async mutate(input: { subjectId: string; sessionId: string; expected: bigint; operation: string; focus: Partial<Focus> }, signal?: AbortSignal) {
    const snapshot = await this.store.read(input.subjectId, input.sessionId, signal);
    if (snapshot.session.closed) throw new ConnectError("Session is closed", Code.FailedPrecondition);
    if (snapshot.session.runtimeRevision !== input.expected) throw new ConnectError("Stale Session revision", Code.Aborted);
    const operation = input.operation;
    let target = snapshot.focuses.find(f => f.focusId === input.focus.focusId);
    const creating = operation === "create";
    if (creating) {
      if (target) throw new ConnectError("Focus already exists", Code.AlreadyExists);
      target = { focusId: randomUUID(), descriptor: input.focus.descriptor ?? "", references: input.focus.references ?? [], state: "SUSPENDED", revision: 0n };
    } else if (!target) throw new ConnectError("Focus not found", Code.NotFound);
    if (!target) throw new ConnectError("Focus not found", Code.NotFound);
    if (target.state === "CLOSED") throw new ConnectError("Focus is closed", Code.FailedPrecondition);
    if (!target.descriptor.trim() || Buffer.byteLength(target.descriptor) > 8192 || target.references.length > 256) throw new ConnectError("Invalid Focus bounds", Code.InvalidArgument);
    const changes: FocusChange[] = [];
    let foreground = snapshot.session.activeFocusId;
    const originalRevision = target.revision;
    const valid=await this.store.validateRefs(input.subjectId,target.references,signal);
    const dropped=target.references.length-valid.length;
    if(creating&&dropped)throw new ConnectError("Focus references must belong to the Subject",Code.InvalidArgument);
    target = { ...target, references: valid };
    switch (operation) {
      case "create": break;
      case "activate":
      case "resume":
      case "switch": {
        const active = snapshot.focuses.find(f => f.focusId === foreground);
        if (active && active.focusId !== target.focusId) changes.push({ focus: { ...active, state: "SUSPENDED", revision: active.revision + 1n }, expectedRevision: active.revision });
        target.state = "ACTIVE";
        foreground = target.focusId;
        break;
      }
      case "suspend":
        target.state = "SUSPENDED";
        if (foreground === target.focusId) foreground = undefined;
        break;
      case "close":
        target.state = "CLOSED";
        if (foreground === target.focusId) foreground = undefined;
        break;
      default: throw new ConnectError("Unknown Focus operation", Code.InvalidArgument);
    }
    target.revision = originalRevision + 1n;
    changes.push({ focus: target, expectedRevision: originalRevision });
    const runtimeRevision = await this.store.write(input.subjectId, input.sessionId, input.expected, changes, foreground, signal);
    return { focus: target, runtimeRevision,degradation:dropped?[{code:"focus_sources_unavailable",detail:`${dropped} unavailable source references were removed during recovery`}]:[] };
  }
  async list(subjectId: string, sessionId: string, signal?: AbortSignal) {
    const snapshot = await this.store.read(subjectId, sessionId, signal);
    return { items: snapshot.focuses, runtimeRevision: snapshot.session.runtimeRevision };
  }
  async get(subjectId: string, sessionId: string, focusId: string, signal?: AbortSignal) {
    const { items, runtimeRevision } = await this.list(subjectId, sessionId, signal);
    const focus = items.find(f => f.focusId === focusId);
    if (!focus) throw new ConnectError("Focus not found", Code.NotFound);
    return { focus, runtimeRevision };
  }
}
