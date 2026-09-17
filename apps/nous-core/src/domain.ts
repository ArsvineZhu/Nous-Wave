export interface Ref { kind: string; value: string }
export interface Evidence { reference?: Ref; supportRole: string }
export interface Focus {
  focusId: string;
  descriptor: string;
  state: "ACTIVE" | "SUSPENDED" | "CLOSED";
  references: Ref[];
  summary?: string;
  revision: bigint;
}
export interface Session {
  sessionId: string;
  subjectId: string;
  runtimeRevision: bigint;
  closed: boolean;
  activeFocusId?: string;
}
export interface RuntimeSnapshot { session: Session; focuses: Focus[] }
export interface FocusChange { focus: Focus; expectedRevision: bigint }
export interface RuntimePort {
  read(subjectId: string, sessionId: string, signal?: AbortSignal): Promise<RuntimeSnapshot>;
  write(subjectId: string, sessionId: string, expected: bigint, changes: FocusChange[], foreground: string | undefined, signal?: AbortSignal): Promise<bigint>;
  validateRefs(subjectId: string, refs: Ref[], signal?: AbortSignal): Promise<Ref[]>;
}
export type Requirement = "REQUIRED" | "PREFERRED" | "OPTIONAL" | "FORBIDDEN";
export interface ConsumerPolicy {
  consumerId: string;
  revision: string;
  memory: Requirement;
  runtime: Requirement;
  resource: Requirement;
  maxItems: number;
  maxTextBytes: number;
  materialize: boolean;
}
export interface Segment {
  segmentId: string;
  text: string;
  semanticRole: string;
  sourceRefs: Ref[];
  evidence: Evidence[];
  authority: string;
  stability: string;
  sourceRevision?: string;
}
export interface Degradation { code: string; detail: string }
export interface Projection {
  projectionId: string;
  consumerId: string;
  sourceRuntimeRevision: bigint;
  segments: Segment[];
  degradation: Degradation[];
}
export interface Cursor { trackId: string; epochId: string; revision: number }
export interface ContextPatch { kind: "RESET" | "APPEND"; cursor: Cursor; projection: Projection }
export function refKey(ref: Ref): string { return `${ref.kind}\0${ref.value}`; }
