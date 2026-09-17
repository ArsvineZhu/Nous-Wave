import Fastify from "fastify";
import multipart from "@fastify/multipart";
import { fastifyConnectPlugin } from "@connectrpc/connect-fastify";
import { Code, ConnectError, type HandlerContext, type ServiceImpl } from "@connectrpc/connect";
import { create } from "@bufbuild/protobuf";
import { timingSafeEqual } from "node:crypto";
import { Readable } from "node:stream";
import { SubjectService, CognitionService, MemoryService, MaterialService } from "@nous-wave/protocol/nous/wave/v1alpha1/services_pb.js";
import { IdentityService, ResolveIdentityResponseSchema } from "@nous-wave/protocol/nous/wave/v1alpha1/identity_pb.js";
import { ResourceService, TopologyService, SystemService } from "@nous-wave/protocol/nous/wave/v1alpha1/management_pb.js";
import { compileNousQL } from "./nousql/compiler.js";
import { FocusSchema, ProjectionRequestSchema, ProjectionSchema, ManagedContextResponseSchema, QueryRequestSchema } from "@nous-wave/protocol/nous/wave/v1alpha1/types_pb.js";
import type { KernelClient } from "./kernel-client.js";
import type { ConsumerPolicy } from "./domain.js";
import { FocusRuntime } from "./cognition/focus.js";
import { ProjectionPlanner } from "./cognition/projection.js";
import { ContextCompiler } from "./cognition/context.js";
import { ModelRuntime } from "./model/runtime.js";
import { ModelMaterialPipeline } from "./model/material.js";
import { ModelService } from "@nous-wave/protocol/nous/wave/v1alpha1/model_pb.js";
import { modelOperations } from "./model/operations.js";

export interface CoreOptions { kernel: KernelClient; token: string; consumers: ConsumerPolicy[]; maxUploadBytes?: number; models?: ModelRuntime }
const options = (context: HandlerContext) => ({ signal: context.signal, timeoutMs: context.timeoutMs() });
export async function createCore(settings: CoreOptions) {
  if (settings.token.length < 32) throw new Error("Core credential must contain at least 32 characters");
  const app = Fastify({ logger: false, bodyLimit: 2 * 1024 * 1024 });
  const kernel = settings.kernel;
  const focuses = new FocusRuntime(kernel);
  const planner = new ProjectionPlanner(new Map(settings.consumers.map(p => [p.consumerId, p])), settings.models);
  const contexts = new ContextCompiler();
  const models = new ModelMaterialPipeline(kernel,settings.models??new ModelRuntime());
  app.addHook("onRequest", async (request, reply) => {
    const origin = request.headers.origin;
    if (origin && origin !== `http://${request.headers.host}`) return reply.code(403).send({ code: "permission_denied", message: "Origin denied" });
    const supplied = Buffer.from(request.headers.authorization ?? "");
    const expected = Buffer.from(`Bearer ${settings.token}`);
    if (supplied.length !== expected.length || !timingSafeEqual(supplied, expected)) return reply.code(401).send({ code: "unauthenticated", message: "Core credential required" });
  });
  async function project(input: Parameters<typeof kernel.authority.buildContributionBatch>[0], context: HandlerContext) {
    const policy = planner.policy(input.consumerId ?? "");
    const request = create(ProjectionRequestSchema, input);
    const snapshot = await kernel.read(request.subjectId, request.sessionId, context.signal);
    if (request.focusId && request.focusId !== snapshot.session.activeFocusId) throw new ConnectError("Projection Focus is not active", Code.FailedPrecondition);
    request.focusId ??= snapshot.session.activeFocusId;
    const active = snapshot.focuses.find(f => f.focusId === request.focusId);
    const activeRefs=await kernel.validateRefs(request.subjectId,active?.references??[],context.signal);
    request.situationRefs.push(...activeRefs.map(r => ({ ...r, $typeName: "nous.wave.v1alpha1.CognitiveRef" as const })));
    request.maxItems = Math.min(request.maxItems || policy.maxItems, policy.maxItems);
    request.maxTextBytes = Math.min(request.maxTextBytes || policy.maxTextBytes, policy.maxTextBytes);
    const queryDegradation=[];
    if(request.query){
      const query=await models.query(create(QueryRequestSchema,{subjectId:request.subjectId,sessionId:request.sessionId,expression:request.query}),options(context));
      request.situationRefs.push(...query.hits.flatMap(h=>h.reference?[h.reference]:[]));queryDegradation.push(...query.degradation);request.query=undefined;
    }
    const batch = await kernel.authority.buildContributionBatch(request, options(context));
    batch.degradation.push(...queryDegradation);
    if(activeRefs.length!==(active?.references.length??0))batch.degradation.push({$typeName:"nous.wave.v1alpha1.Degradation",code:"focus_sources_unavailable",detail:"Some Focus sources are no longer available"});
    const projection=await planner.build({
      projectionId:batch.projectionId,consumerId:batch.consumerId,sourceRuntimeRevision:batch.sourceRuntimeRevision,
      degradation:batch.degradation.map(d=>({code:d.code,detail:d.detail})),
      segments:batch.segments.map(s=>({segmentId:s.segmentId,text:s.text,semanticRole:s.semanticRole,authority:s.authority,stability:s.stability,sourceRevision:s.sourceRevision,
        sourceRefs:s.sourceRefs.map(r=>({kind:r.kind,value:r.value})),evidence:s.evidence.map(e=>({supportRole:e.supportRole,reference:e.reference?{kind:e.reference.kind,value:e.reference.value}:undefined}))})),
    }, request, context.signal);
    const after=await kernel.read(request.subjectId,request.sessionId,context.signal);
    if(after.session.runtimeRevision!==snapshot.session.runtimeRevision)throw new ConnectError("Session changed during projection",Code.Aborted);
    return projection;
  }
  const cognition: ServiceImpl<typeof CognitionService> = {
    openSession: (r,c) => kernel.authority.openSession(r,options(c)),
    getSession: (r,c) => kernel.authority.getSession(r,options(c)),
    listSessions: (r,c) => kernel.authority.listSessions(r,options(c)),
    closeSession: (r,c) => kernel.authority.closeSession(r,options(c)),
    recordObservation: (r,c) => kernel.authority.recordObservation(r,options(c)),
    query: async (r,c) => {
      let boundQuery: string|undefined;
      if(r.nousql!==undefined){
        if(r.expression)throw new ConnectError("Supply either NousQL or typed expression",Code.InvalidArgument);
        const compiled=await compileNousQL(r.nousql,async(kind,locator)=>{
          const result=await kernel.authority.resolveIdentity({subjectId:r.subjectId,kind,locator:{case:locator.kind==="name"?"name":"lexicalRef",value:locator.value}},options(c));
          if(result.status!=="BOUND"||!result.candidates[0]?.canonical)throw new ConnectError(result.status,Code.InvalidArgument,undefined,[{desc:ResolveIdentityResponseSchema,value:result}]);
          return {canonical:result.candidates[0].canonical,lexicalRef:result.candidates[0].lexicalRef};
        });
        r.expression=compiled.expression;r.nousql=undefined;boundQuery=compiled.boundCanonical;
      }
      const result=await models.query(r,options(c));
      for(const hit of result.hits){
        if(hit.reference&&hit.reference.kind!=="external_object"){
          const binding=await kernel.authority.bindIdentity({subjectId:r.subjectId,canonical:hit.reference},options(c));
          hit.lexicalRef=binding.lexicalRef;
        }
      }
      result.boundQuery=boundQuery;
      return result;
    },
    reportUse: (r,c) => kernel.authority.reportUse(r,options(c)),
    mutateFocus: async (r,c) => {
      if (!r.focus) throw new ConnectError("Focus input required",Code.InvalidArgument);
      const result = await focuses.mutate({ subjectId:r.subjectId,sessionId:r.sessionId,expected:r.expectedRuntimeRevision,operation:r.operation,
        focus:{focusId:r.focus.focusId,descriptor:r.focus.descriptor,references:r.focus.references} },c.signal);
      return { focus:create(FocusSchema,result.focus),runtimeRevision:result.runtimeRevision,degradation:result.degradation };
    },
    getFocus: async (r,c) => { const result=await focuses.get(r.subjectId,r.sessionId,r.focusId,c.signal);return {...result,focus:create(FocusSchema,result.focus)}; },
    listFocuses: async (r,c) => { const result=await focuses.list(r.subjectId,r.sessionId,c.signal);return {...result,items:result.items.map(f=>create(FocusSchema,f))}; },
    buildProjection: async(r,c)=>create(ProjectionSchema,await project(r,c)),
    buildManagedContext: async (r,c) => {
      if (!r.projection) throw new ConnectError("Projection request required",Code.InvalidArgument);
      const projection=await project(r.projection,c);
      const session=await kernel.authority.getSession({subjectId:r.projection.subjectId,id:r.projection.sessionId},options(c));
      if(session.runtimeRevision!==projection.sourceRuntimeRevision)throw new ConnectError("Session changed before context synchronization",Code.Aborted);
      const policy=planner.policy(r.projection.consumerId);
      return create(ManagedContextResponseSchema,contexts.compile({subjectId:r.projection.subjectId,sessionId:r.projection.sessionId,consumerId:r.projection.consumerId,focusId:r.projection.focusId??session.activeFocusId},`${policy.revision}:${r.projection.maxItems}:${r.projection.maxTextBytes}`,projection,r.knownCursor));
    },
  };
  const subjects: ServiceImpl<typeof SubjectService> = {
    createSubject:(r,c)=>kernel.authority.createSubject(r,options(c)),getSubject:(r,c)=>kernel.authority.getSubject(r,options(c)),listSubjects:(r,c)=>kernel.authority.listSubjects(r,options(c)),
    getCharacterSeed:(r,c)=>kernel.authority.getCharacterSeed(r,options(c)),reviseCharacterSeed:(r,c)=>kernel.authority.reviseCharacterSeed(r,options(c)),
  };
  const memories: ServiceImpl<typeof MemoryService> = {
    consolidateMemory:(r,c)=>kernel.authority.consolidateMemory(r,options(c)),
    getMemory:(r,c)=>kernel.authority.getMemory(r,options(c)),listMemories:(r,c)=>kernel.authority.listMemories(r,options(c)),
    getMemoryRevision:(r,c)=>kernel.authority.getMemoryRevision(r,options(c)),listMemoryRevisions:(r,c)=>kernel.authority.listMemoryRevisions(r,options(c)),
    formMemory:(r,c)=>kernel.authority.formMemory(r,options(c)),reviseMemory:(r,c)=>kernel.authority.reviseMemory(r,options(c)),
    suppressMemory:(r,c)=>kernel.authority.suppressMemory(r,options(c)),restoreMemory:(r,c)=>kernel.authority.restoreMemory(r,options(c)),purgeMemory:(r,c)=>kernel.authority.purgeMemory(r,options(c)),
  };
  const material: ServiceImpl<typeof MaterialService> = {
    getOccurrence:(r,c)=>kernel.authority.getOccurrence(r,options(c)),getSourceRegion:(r,c)=>kernel.authority.getSourceRegion(r,options(c)),getDerivedRepresentation:(r,c)=>kernel.authority.getDerivedRepresentation(r,options(c)),
    getArtifact:(r,c)=>kernel.authority.getArtifact(r,options(c)),listArtifacts:(r,c)=>kernel.authority.listArtifacts(r,options(c)),materializeEvidence:(r,c)=>kernel.authority.materializeEvidence(r,options(c)),
  };
  const identities:ServiceImpl<typeof IdentityService>={bindIdentity:(r,c)=>kernel.authority.bindIdentity(r,options(c)),resolveIdentity:(r,c)=>kernel.authority.resolveIdentity(r,options(c))};
  const resources:ServiceImpl<typeof ResourceService>={putResource:(r,c)=>kernel.authority.putResource(r,options(c)),getResource:(r,c)=>kernel.authority.getResource(r,options(c)),listResources:(r,c)=>kernel.authority.listResources(r,options(c)),removeResource:(r,c)=>kernel.authority.removeResource(r,options(c))};
  const topology:ServiceImpl<typeof TopologyService>={createTag:(r,c)=>kernel.authority.createTag(r,options(c)),getTag:(r,c)=>kernel.authority.getTag(r,options(c)),listTags:(r,c)=>kernel.authority.listTags(r,options(c)),createAnchor:(r,c)=>kernel.authority.createAnchor(r,options(c)),getAnchor:(r,c)=>kernel.authority.getAnchor(r,options(c)),listAnchors:(r,c)=>kernel.authority.listAnchors(r,options(c)),createAssociation:(r,c)=>kernel.authority.createAssociation(r,options(c)),getNeighborhood:(r,c)=>kernel.authority.getNeighborhood(r,options(c)),rebindEntity:(r,c)=>kernel.authority.rebindEntity(r,options(c))};
  const system:ServiceImpl<typeof SystemService>={
    getStatus:async(r,c)=>{try{return await kernel.authority.getStatus(r,options(c));}catch(error){if(c.signal.aborted)throw error;return {components:[{name:"kernel",state:"UNAVAILABLE",detail:"Kernel unavailable"}]};}},
    getCapabilities:async(r,c)=>{const status=await kernel.authority.getStatus(r,options(c));return {components:[...status.components,{name:"model.steward",state:settings.models?.readiness??"NOT_CONFIGURED"}]};},
    getProjectionStatus:(r,c)=>kernel.authority.getProjectionStatus(r,options(c)),
    getEffectiveConfig:()=>({entries:settings.consumers.map(p=>({key:`consumer.${p.consumerId}`,value:JSON.stringify(p),source:"CONFIG",owner:"host",restartRequired:true,editable:false}))}),
  };
  await app.register(fastifyConnectPlugin,{routes:router=>{router.service(SubjectService,subjects);router.service(CognitionService,cognition);router.service(MemoryService,memories);router.service(MaterialService,material);router.service(IdentityService,identities);router.service(ResourceService,resources);router.service(TopologyService,topology);router.service(SystemService,system);router.service(ModelService,modelOperations(kernel,settings.models??new ModelRuntime()));},grpc:false,grpcWeb:false,readMaxBytes:2*1024*1024,writeMaxBytes:4*1024*1024});
  await app.register(multipart,{limits:{files:1,fields:0,fileSize:settings.maxUploadBytes??8*1024**3}});
  app.post<{Params:{subjectId:string}}>("/artifacts/:subjectId",async(request,reply)=>{
    const file=await request.file();
    if(!file) return reply.code(400).send({code:"invalid_argument",message:"One file required"});
    async function* chunks(){
      yield {part:{case:"header" as const,value:{subjectId:request.params.subjectId,mediaType:file!.mimetype}}};
      for await(const chunk of file!.file){
        const bytes=Buffer.from(chunk);
        for(let offset=0;offset<bytes.length;offset+=262144)yield {part:{case:"content" as const,value:bytes.subarray(offset,offset+262144)}};
      }
      if(file!.file.truncated)throw new ConnectError("Upload limit exceeded",Code.ResourceExhausted);
    }
    const value=await kernel.artifacts.uploadArtifact(chunks(),{timeoutMs:0});
    return {artifactId:value.artifactId,subjectId:value.subjectId,contentHash:value.contentHash,byteLength:value.byteLength.toString(),mediaType:value.mediaType};
  });
  app.get<{Params:{subjectId:string;artifactId:string}}>("/artifacts/:subjectId/:artifactId",async(request,reply)=>{
    const meta=await kernel.authority.getArtifact({subjectId:request.params.subjectId,id:request.params.artifactId});
    const length=Number(meta.byteLength);let start=0;let end=length;
    if(request.headers.range){
      const match=/^bytes=(\d*)-(\d*)$/.exec(request.headers.range);
      if(!match || (!match[1]&&!match[2]))return reply.code(416).header("content-range",`bytes */${length}`).send();
      if(!match[1])start=Math.max(0,length-Number(match[2]));else{start=Number(match[1]);if(match[2])end=Math.min(length,Number(match[2])+1);}
      if(!Number.isSafeInteger(start)||!Number.isSafeInteger(end)||start>=end)return reply.code(416).header("content-range",`bytes */${length}`).send();
      reply.code(206).header("content-range",`bytes ${start}-${end-1}/${length}`);
    }
    const stream=kernel.artifacts.downloadArtifact({subjectId:request.params.subjectId,artifactId:request.params.artifactId,start:BigInt(start),end:BigInt(end)},{timeoutMs:0});
    async function* bytes(){for await(const chunk of stream)yield Buffer.from(chunk.content);}
    return reply.header("accept-ranges","bytes").header("content-type",meta.mediaType).header("content-length",end-start).send(Readable.from(bytes()));
  });
  return app;
}
