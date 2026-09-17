import { createHash, randomUUID } from "node:crypto";
import { Code, ConnectError } from "@connectrpc/connect";
import type { ContextPatch, Cursor, Projection, Segment } from "../domain.js";

interface Track { cursor: Cursor; segments: Segment[]; profile: string }
const digest = (value: unknown) => createHash("sha256").update(JSON.stringify(value)).digest("hex");

export class ContextCompiler {
  private readonly tracks = new Map<string, Track>();
  constructor(private readonly maxTracks = 256) {}

  compile(key: { subjectId: string; sessionId: string; focusId?: string; consumerId: string }, profile: string, projection: Projection, known?: Cursor): ContextPatch {
    const trackId = digest([key.subjectId, key.sessionId, key.focusId ?? "", key.consumerId]);
    const previous = this.tracks.get(trackId);
    const synchronized = previous && known?.trackId === trackId && known.epochId === previous.cursor.epochId && known.revision === previous.cursor.revision && previous.profile === profile;
    const appendable = synchronized && previous.segments.length <= projection.segments.length && previous.segments.every((segment, index) => digest(segment) === digest(projection.segments[index]));
    if (!appendable) {
      const cursor = { trackId, epochId: randomUUID(), revision: 1 };
      this.save(trackId, { cursor, segments: structuredClone(projection.segments), profile });
      return { kind: "RESET", cursor, projection };
    }
    const delta = projection.segments.slice(previous.segments.length);
    if (previous.cursor.revision === 0xffffffff) throw new ConnectError("Context revision exhausted", Code.ResourceExhausted);
    const cursor = { ...previous.cursor, revision: previous.cursor.revision + (delta.length > 0 ? 1 : 0) };
    this.save(trackId, { cursor, segments: structuredClone(projection.segments), profile });
    return { kind: "APPEND", cursor, projection: { ...projection, segments: delta } };
  }
  private save(key: string, track: Track) {
    this.tracks.delete(key);
    this.tracks.set(key, track);
    if (this.tracks.size > this.maxTracks) this.tracks.delete(this.tracks.keys().next().value!);
  }
}
