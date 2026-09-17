import { afterAll, beforeAll, expect, it, vi } from "vitest";
import { mkdir, mkdtemp, readFile, writeFile } from "node:fs/promises";
import { resolve, join } from "node:path";
import { randomBytes, randomUUID } from "node:crypto";
import { startKernel, type KernelProcess } from "../src/process.js";
import { createCore } from "../src/server.js";
import { connectNous } from "../../../packages/client/src/node.js";
import { ModelRuntime } from "../src/model/runtime.js";

let kernel: KernelProcess;
let app: Awaited<ReturnType<typeof createCore>>;
let client: ReturnType<typeof connectNous>;
let address: string;
const token = randomBytes(32).toString("hex");
const now = { seconds: BigInt(Math.floor(Date.now() / 1000)), nanos: 0 };
const models=ModelRuntime.withRoles({embedding:"test-embedding",formation:"controlled-test",interpretation:"controlled-test"});
vi.spyOn(models,"embedding").mockImplementation(async()=>[1,0,0]);
vi.spyOn(models,"form").mockImplementation(async(text)=>({text:`Derived: ${text}`,semanticRole:"episode"}));
vi.spyOn(models,"interpret").mockImplementation(async(bytes)=>new TextDecoder().decode(bytes));
beforeAll(async () => {
  const local = resolve(".local");
  await mkdir(local, { recursive: true });
  const root = await mkdtemp(join(local, "rebase-acceptance-"));
  const original = await readFile("config.example.toml", "utf8");
  const config = original.replace(/name = "nous_wave_\d+"/, 'name = "nous_rebase_acceptance"')+`
[stored_embedding.space]
space_hash="test-space"
model_identity="test-embedding"
weights_revision="test"
task="retrieval"
input_representation="text"
preprocessing_identity="identity"
preprocessing_revision="1"
dimension=3
normalization="none"
output_semantics="dense_similarity"
[stored_embedding.producer]
signature_hash="test-producer"
provider_class="host-model"
operation="text_embedding"
implementation="controlled-test"
model_identity="test-embedding"
model_revision="test"
preprocessing_identity="identity"
preprocessing_revision="1"
config_digest="test"
`;
  await writeFile(join(root, "kernel.toml"), config);
  kernel = await startKernel(resolve(`target/debug/nous-kernel${process.platform === "win32" ? ".exe" : ""}`), join(root, "kernel.toml"));
  app = await createCore({ kernel: kernel.client, token, models, consumers: [{ consumerId: "test", revision: "1", memory: "PREFERRED", runtime: "OPTIONAL", resource: "OPTIONAL", maxItems: 16, maxTextBytes: 8192, materialize: true }] });
  address = await app.listen({ host: "127.0.0.1", port: 0 });
  client = connectNous(address, token);
}, 120_000);
afterAll(async () => { await app?.close(); await kernel?.stop(); }, 20_000);

it("runs Client → Core → Kernel contracts with durable retry, head fencing and Focus CAS", async () => {
  const subject = await client.subjects.create({ characterSeed: { text: "Independent Subject", mediaType: "text/plain" } });
  expect(subject).not.toHaveProperty("$typeName");
  const subjectId = subject.subjectId;
  const session = await client.cognition.openSession({ subjectId });
  const sessionId = session.sessionId;
  const observation = { subjectId, sessionId, requestId: randomUUID(), sourceClass: "message", observedAt: now, admit: true,
    material: { case: "inlineText" as const, value: { text: "Alice studies Rust ownership at university", mediaType: "text/plain" } } };
  const [first, replay] = await Promise.all([client.cognition.observe(observation), client.cognition.observe(observation)]);
  expect(first.occurrenceId).toBe(replay.occurrenceId);
  await expect(client.cognition.observe({ ...observation, material: { case: "inlineText", value: { text: "different", mediaType: "text/plain" } } })).rejects.toMatchObject({ code: 10 });
  const evidence = [{ reference: { kind: "occurrence", value: first.occurrenceId }, supportRole: "direct" }];
  const memory = await client.memory.form({ subjectId, input: { memoryClass: "specific", semanticRole: "episode", text: "Alice studies Rust ownership", epistemicClass: "observed", observedAt: now, evidence } });
  const edited = await client.memory.revise({ subjectId, memoryId: memory.memoryId, expectedEtag: memory.etag, input: { text: "Alice studies Rust ownership in school", semanticRole: "episode", epistemicClass: "observed", evidence } });
  await expect(client.memory.suppress({ subjectId, memoryId: memory.memoryId, expectedEtag: memory.etag })).rejects.toMatchObject({ code: 10 });
  const suppressed = await client.memory.suppress({ subjectId, memoryId: memory.memoryId, expectedEtag: edited.etag });
  const restored = await client.memory.restore({ subjectId, memoryId: memory.memoryId, expectedEtag: suppressed.etag });
  expect(restored.etag).not.toBe(edited.etag);
  const state = await client.cognition.getSession({ subjectId, id: sessionId });
  const focus = await client.cognition.mutateFocus({ subjectId, sessionId, expectedRuntimeRevision: state.runtimeRevision, operation: "create", focus: { descriptor: "Study Rust", references: [{ kind: "memory", value: memory.memoryId }] } });
  const active = await client.cognition.mutateFocus({ subjectId, sessionId, expectedRuntimeRevision: focus.runtimeRevision, operation: "activate", focus: { focusId: focus.focus!.focusId } });
  await expect(client.cognition.mutateFocus({ subjectId, sessionId, expectedRuntimeRevision: focus.runtimeRevision, operation: "suspend", focus: { focusId: focus.focus!.focusId } })).rejects.toMatchObject({ code: 10 });
  expect(active.focus!.state).toBe("ACTIVE");
  const projection = { subjectId, sessionId, consumerId: "test", maxItems: 8, maxTextBytes: 8192 };
  const context = await client.cognition.managedContext({ projection });
  expect(context.kind).toBe("RESET");
  expect(context.projection!.segments.some(s => s.text.includes("Rust"))).toBe(true);
  const same = await client.cognition.managedContext({ projection, knownCursor: context.cursor });
  expect(same.kind).toBe("APPEND");
  expect(same.projection!.segments).toHaveLength(0);
  await client.cognition.reportUse({ subjectId, sessionId, consumerId: "test", events: [{ reference: { kind: "memory", value: memory.memoryId }, kind: "exposed" }] });
  const unauthorized = await fetch(`${address}/artifacts/${subjectId}/${first.artifactId}`);
  expect(unauthorized.status).toBe(401);
  const form = new FormData(); form.set("file", new Blob(["abcdef"], { type: "text/plain" }), "test.txt");
  const upload = await fetch(`${address}/artifacts/${subjectId}`, { method: "POST", headers: { authorization: `Bearer ${token}` }, body: form });
  expect(upload.status).toBe(200);
  const uploaded = await upload.json() as { artifactId: string };
  const ranged = await fetch(`${address}/artifacts/${subjectId}/${uploaded.artifactId}`, { headers: { authorization: `Bearer ${token}`, range: "bytes=1-3" } });
  expect(ranged.status).toBe(206); expect(await ranged.text()).toBe("bcd");
}, 60_000);

it("keeps lexical identity exact and tombstoned across Memory purge",async()=>{
  const subject=await client.subjects.create({characterSeed:{text:"Identity research",mediaType:"text/plain"}});
  const subjectId=subject.subjectId;
  const a=await client.identity.bind({subjectId,canonical:{kind:"entity",value:"entity:test:alice1"},displayName:"Alice"});
  const b=await client.identity.bind({subjectId,canonical:{kind:"entity",value:"entity:test:alice2"},displayName:"Alice"});
  expect(a.lexicalRef).not.toBe(b.lexicalRef);
  const ambiguous=await client.identity.resolve({subjectId,kind:"entity",locator:{case:"name",value:"Alice"}});
  expect(ambiguous.status).toBe("AMBIGUOUS_REFERENCE");expect(ambiguous.candidates).toHaveLength(2);
  await expect(client.cognition.query({subjectId,nousql:'@e("Alice")'})).rejects.toMatchObject({code:3});
  const observation=await client.cognition.observe({subjectId,sourceClass:"file",observedAt:now,material:{case:"inlineText",value:{text:"Research on Rust ownership",mediaType:"text/plain"}}});
  const memory=await client.memory.form({subjectId,input:{memoryClass:"specific",semanticRole:"episode",text:"Research on Rust ownership",epistemicClass:"observed",observedAt:now,evidence:[{reference:{kind:"occurrence",value:observation.occurrenceId},supportRole:"direct"}]}});
  const binding=await client.identity.bind({subjectId,canonical:{kind:"memory",value:memory.memoryId}});
  const results=await client.cognition.recall(subjectId,`@ref(${binding.lexicalRef}) $memory`);
  expect(results.hits.some(h=>h.ref===binding.lexicalRef)).toBe(true);
  await client.memory.purge({subjectId,memoryId:memory.memoryId,expectedEtag:memory.etag});
  const tombstone=await client.identity.resolve({subjectId,kind:"memory",locator:{case:"lexicalRef",value:binding.lexicalRef}});
  expect(tombstone.status).toBe("REFERENCE_TOMBSTONED");
},30_000);

it("exposes bounded topology and truthful unresolved Resource Authority",async()=>{
  const {subjectId}=await client.subjects.create({characterSeed:{text:"Resource research",mediaType:"text/plain"}});
  const tag=await client.topology.createTag({subjectId,tag:{label:"school",origin:"explicit"}});
  const anchor=await client.topology.createAnchor({subjectId,anchor:{label:"college",description:"College years",confirmed:true,supports:[{reference:{kind:"tag",value:tag.tagId},role:"landmark"}]}});
  expect((await client.topology.getAnchor({subjectId,id:anchor.anchorId})).confirmed).toBe(true);
  expect((await client.topology.listTags({subjectId,page:{pageSize:1}})).items[0]?.tagId).toBe(tag.tagId);
  const neighborhood=await client.topology.neighborhood({subjectId,root:{kind:"anchor",value:anchor.anchorId},maxDepth:1,maxNodes:4});
  expect(neighborhood.nodes).toHaveLength(1);
  await client.resources.put({subjectId,descriptor:{resourceRef:"resource:test:calendar",displayLabel:"calendar",authorityClass:"external",accessCostClass:"low",readiness:"ready",queryDimensions:["time"],modalities:["text"]}});
  const query=await client.cognition.query({subjectId,nousql:'@r("calendar") $current(required)'});
  expect(query.status).toBe("partial");expect(query.resourceActions).toHaveLength(1);expect(query.resourceActions[0]?.currentAuthority).toBe(false);
  expect((await client.system.status({})).components.length).toBeGreaterThan(0);
  await client.system.projections({subjectId});
},30_000);

it("accepts bounded Host model material with evidence and embedding-space checks",async()=>{
  const {subjectId}=await client.subjects.create({characterSeed:{text:"Model research",mediaType:"text/plain"}});
  const observed=await client.cognition.observe({subjectId,sourceClass:"file",observedAt:now,material:{case:"inlineText",value:{text:"A textual source for model interpretation",mediaType:"text/plain"}}});
  const formed=await client.model.formFromObservation({subjectId,sourceId:observed.occurrenceId});
  expect(formed.memory?.epistemicClass).toBe("derived");expect(formed.memory?.evidence[0]?.reference?.value).toBe(observed.occurrenceId);
  const interpreted=await client.model.interpretSource({subjectId,sourceId:observed.sourceRegionId!});
  expect(interpreted.representation?.sourceRegionId).toBe(observed.sourceRegionId);
  expect(interpreted.representation?.producerSignature).toMatch(/^[a-f0-9]{64}$/);
  const embeddings=await client.model.prepareEmbeddings({subjectId,limit:64});
  expect(embeddings.degradation).toHaveLength(0);expect(embeddings.committed).toBeGreaterThan(0);
  const query=await client.cognition.query({subjectId,nousql:'"unrelated vocabulary" $memory'});
  expect(query.hits.some(h=>h.evidenceFamilies.includes("semantic_dense"))).toBe(true);
  await expect(kernel.client.modelMaterial.commitEmbedding({subjectId,reference:{kind:"memory",value:formed.memory!.memoryId},material:{text:formed.memory!.text,spaceHash:"wrong",producerHash:"wrong",vector:[1,2]}})).rejects.toMatchObject({code:3});
},30_000);
