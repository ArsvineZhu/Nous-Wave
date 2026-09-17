import { Code, ConnectError, type CallOptions } from "@connectrpc/connect";
import { create } from "@bufbuild/protobuf";
import { QueryEmbeddingSchema, type QueryEmbedding } from "@nous-wave/protocol/nous/wave/kernel/v1alpha1/model_pb.js";
import type { QueryExpr, QueryRequest } from "@nous-wave/protocol/nous/wave/v1alpha1/types_pb.js";
import type { KernelClient } from "../kernel-client.js";
import { ModelRuntime } from "./runtime.js";

export class ModelMaterialPipeline {
  constructor(private readonly kernel:KernelClient, private readonly models:ModelRuntime){}
  async query(input:QueryRequest,options:CallOptions={}){
    const material:QueryEmbedding[]=[];
    let failure:string|undefined;
    if(this.models.embeddingModel){
      try{
        const config=await this.kernel.modelMaterial.getEmbeddingConfig({},options);
        const texts=new Set<string>();
        const collect=(node:QueryExpr)=>{
          const text=node.cues.map(c=>c.cue.case==="text"||c.cue.case==="concept"?c.cue.value:"").filter(Boolean).join(" ").trim();
          if(text)texts.add(text);
          node.children.forEach(collect);
          for(const p of node.modifiers?.preferences??[]){if(p.cue&&(p.cue.cue.case==="text"||p.cue.cue.case==="concept"))texts.add(p.cue.cue.value);}
        };
        if(input.expression)collect(input.expression);
        if(texts.size>64)throw new ConnectError("Query material bound exceeded",Code.ResourceExhausted);
        for(const text of texts){
          const vector=await this.models.embedding(text,config.model,options.signal??undefined);
          material.push(create(QueryEmbeddingSchema,{text,spaceHash:config.spaceHash,producerHash:config.producerHash,vector}));
        }
      }catch(error){if(options.signal?.aborted)throw error;failure=error instanceof Error?error.message:"Embedding unavailable";}
    }
    const result=await this.kernel.authority.query({query:input,embeddings:material},options);
    if(failure){result.degradation.push({$typeName:"nous.wave.v1alpha1.Degradation",code:"query_embedding_unavailable",detail:failure});if(result.status==="complete")result.status="degraded";}
    return result;
  }
  async prepare(subjectId:string,limit:number,options:CallOptions={}){
    const needs=await this.kernel.modelMaterial.listEmbeddingNeeds({subjectId,limit},options);
    if(!needs.config)throw new ConnectError("Embedding configuration unavailable",Code.FailedPrecondition);
    let committed=0;
    for(const need of needs.needs){
      try{
        const vector=await this.models.embedding(need.text,needs.config.model,options.signal??undefined);
        await this.kernel.modelMaterial.commitEmbedding({subjectId,reference:need.reference,material:{text:need.text,spaceHash:needs.config.spaceHash,producerHash:needs.config.producerHash,vector}},options);
        committed++;
      }catch(error){if(options.signal?.aborted)throw error;return {committed,degradation:[{code:"embedding_batch_incomplete",detail:error instanceof Error?error.message:"Embedding material failed"}]};}
    }
    return {committed,degradation:[]};
  }
}
