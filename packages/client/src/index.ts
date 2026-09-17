import { createClient, ConnectError, type CallOptions, type Transport } from "@connectrpc/connect";
import { SubjectService, CognitionService, MemoryService, MaterialService } from "@nous-wave/protocol/nous/wave/v1alpha1/services_pb.js";
import { IdentityService } from "@nous-wave/protocol/nous/wave/v1alpha1/identity_pb.js";
import { ResourceService, TopologyService, SystemService } from "@nous-wave/protocol/nous/wave/v1alpha1/management_pb.js";
import { ModelService } from "@nous-wave/protocol/nous/wave/v1alpha1/model_pb.js";

// Transport metadata is removed at the official Client boundary.
type Data<T> = T extends Uint8Array ? Uint8Array : T extends readonly (infer I)[] ? Data<I>[] : T extends object ? { [K in keyof T as K extends `$${string}` ? never : K]: Data<T[K]> } : T;
export interface RequestOptions { signal?: AbortSignal; timeoutMs?: number }
export class NousError extends Error {
  readonly code: number;
  readonly details: readonly unknown[];
  constructor(error: ConnectError) {
    super(error.rawMessage, { cause: error });
    this.name = "NousError";
    this.code = error.code;
    this.details = error.details;
  }
}
function plain<T>(value: T): Data<T> {
  if (value instanceof Uint8Array) return value as Data<T>;
  if (Array.isArray(value)) return value.map(plain) as Data<T>;
  if (value !== null && typeof value === "object") return Object.fromEntries(Object.entries(value).filter(([key]) => !key.startsWith("$")).map(([key, val]) => [key, plain(val)])) as Data<T>;
  return value as Data<T>;
}
function call<I, O>(method: (input: I, options?: CallOptions) => Promise<O>) {
  return async (input: I, options?: RequestOptions): Promise<Data<O>> => {
    try { return plain(await method(input, options)); }
    catch (error) { if (error instanceof ConnectError) throw new NousError(error); throw error; }
  };
}
export function createNousClient(transport: Transport) {
  const subjects = createClient(SubjectService, transport);
  const cognition = createClient(CognitionService, transport);
  const memory = createClient(MemoryService, transport);
  const material = createClient(MaterialService, transport);
  const identity = createClient(IdentityService, transport);
  const resources=createClient(ResourceService,transport);
  const topology=createClient(TopologyService,transport);
  const system=createClient(SystemService,transport);
  const model=createClient(ModelService,transport);
  return {
    identity: { bind: call(identity.bindIdentity), resolve: call(identity.resolveIdentity) },
    model:{formFromObservation:call(model.formFromObservation),interpretSource:call(model.interpretSource),prepareEmbeddings:call(model.prepareEmbeddings)},
    resources:{put:call(resources.putResource),get:call(resources.getResource),list:call(resources.listResources),remove:call(resources.removeResource)},
    topology:{createTag:call(topology.createTag),getTag:call(topology.getTag),listTags:call(topology.listTags),createAnchor:call(topology.createAnchor),getAnchor:call(topology.getAnchor),listAnchors:call(topology.listAnchors),associate:call(topology.createAssociation),neighborhood:call(topology.getNeighborhood),rebindEntity:call(topology.rebindEntity)},
    system:{status:call(system.getStatus),capabilities:call(system.getCapabilities),config:call(system.getEffectiveConfig),projections:call(system.getProjectionStatus)},
    subjects: { create: call(subjects.createSubject), get: call(subjects.getSubject), list: call(subjects.listSubjects), seed: call(subjects.getCharacterSeed), reviseSeed: call(subjects.reviseCharacterSeed) },
    cognition: {
      openSession: call(cognition.openSession), getSession: call(cognition.getSession), listSessions: call(cognition.listSessions), closeSession: call(cognition.closeSession),
      observe: call(cognition.recordObservation), query: call(cognition.query), reportUse: call(cognition.reportUse),
      recall: async (subjectId:string,nousql:string,options?:RequestOptions) => {
        const result=await call(cognition.query)({subjectId,nousql},options);
        return {status:result.status,boundQuery:result.boundQuery,degradation:result.degradation,
          hits:result.hits.map(h=>({ref:h.lexicalRef,text:h.text,authority:h.authority,evidenceFamilies:h.evidenceFamilies}))};
      },
      mutateFocus: call(cognition.mutateFocus), getFocus: call(cognition.getFocus), listFocuses: call(cognition.listFocuses),
      project: call(cognition.buildProjection), managedContext: call(cognition.buildManagedContext),
    },
    memory: { get: call(memory.getMemory), list: call(memory.listMemories), revision: call(memory.getMemoryRevision), history: call(memory.listMemoryRevisions),
      form: call(memory.formMemory), revise: call(memory.reviseMemory), suppress: call(memory.suppressMemory), restore: call(memory.restoreMemory), purge: call(memory.purgeMemory),consolidate:call(memory.consolidateMemory) },
    material: { getArtifact: call(material.getArtifact), listArtifacts: call(material.listArtifacts), materialize: call(material.materializeEvidence),occurrence:call(material.getOccurrence),sourceRegion:call(material.getSourceRegion),representation:call(material.getDerivedRepresentation) },
  };
}
export type NousClient = ReturnType<typeof createNousClient>;
