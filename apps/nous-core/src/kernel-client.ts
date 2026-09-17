import { create, fromBinary, toBinary } from "@bufbuild/protobuf";
import { Code, ConnectError, createClient, type Transport } from "@connectrpc/connect";
import { createGrpcTransport } from "@connectrpc/connect-node";
import { AuthorityService, RuntimeStoreService, ArtifactStreamService } from "@nous-wave/protocol/nous/wave/kernel/v1alpha1/kernel_pb.js";
import { ModelMaterialService } from "@nous-wave/protocol/nous/wave/kernel/v1alpha1/model_pb.js";
import { FocusSchema } from "@nous-wave/protocol/nous/wave/v1alpha1/types_pb.js";
import { Health } from "@nous-wave/protocol/grpc/health/v1/health_pb.js";
import type { Focus, FocusChange, Ref, RuntimePort, RuntimeSnapshot } from "./domain.js";

export class KernelClient implements RuntimePort {
  readonly authority;
  readonly runtime;
  readonly artifacts;
  readonly health;
  readonly modelMaterial;
  constructor(transport: Transport) {
    this.authority = createClient(AuthorityService, transport);
    this.runtime = createClient(RuntimeStoreService, transport);
    this.artifacts = createClient(ArtifactStreamService, transport);
    this.health = createClient(Health, transport);
    this.modelMaterial = createClient(ModelMaterialService, transport);
  }
  static connect(endpoint: string, token: string) {
    return new KernelClient(createGrpcTransport({ baseUrl: endpoint, defaultTimeoutMs: 30_000, interceptors: [next => async request => {
      request.header.set("authorization", `Bearer ${token}`);
      return next(request);
    }] }));
  }
  async read(subjectId: string, sessionId: string, signal?: AbortSignal): Promise<RuntimeSnapshot> {
    const result = await this.runtime.readRuntime({ subjectId, sessionId, ownerKind: "focus" }, { signal });
    if (!result.session) throw new ConnectError("Missing runtime Session", Code.DataLoss);
    const focuses: Focus[] = result.checkpoints.map(record => {
      if (record.schemaVersion !== 1) throw new ConnectError("Unsupported Focus checkpoint version", Code.FailedPrecondition);
      const f = fromBinary(FocusSchema, record.payload);
      if (!["ACTIVE", "SUSPENDED", "CLOSED"].includes(f.state) || f.focusId !== record.ownerKey || f.revision !== record.revision) throw new ConnectError("Invalid Focus checkpoint", Code.DataLoss);
      return { focusId: f.focusId, descriptor: f.descriptor, state: f.state as Focus["state"], references: f.references.map(({kind,value}) => ({kind,value})), revision: f.revision, ...(f.summary === undefined ? {} : { summary: f.summary }) };
    });
    return { session: result.session, focuses };
  }
  async write(subjectId: string, sessionId: string, expected: bigint, changes: FocusChange[], foreground: string | undefined, signal?: AbortSignal) {
    const result = await this.runtime.mutateRuntime({ subjectId, sessionId, expectedRuntimeRevision: expected, changeForeground: true, foregroundKey: foreground,
      checkpoints: changes.map(({ focus, expectedRevision }) => ({ ownerKind: "focus", ownerKey: focus.focusId, schemaVersion: 1, revision: expectedRevision, payload: toBinary(FocusSchema, create(FocusSchema, focus)) })),
    }, { signal });
    return result.runtimeRevision;
  }
  async validateRefs(subjectId: string, refs: Ref[], signal?: AbortSignal): Promise<Ref[]> {
    if (refs.length > 256) throw new ConnectError("Too many references", Code.ResourceExhausted);
    const result=await this.runtime.checkReferences({subjectId,references:refs},{signal});
    return result.valid.map(r=>({kind:r.kind,value:r.value}));
  }
}
