import { create } from "@bufbuild/protobuf";
import { Code, ConnectError } from "@connectrpc/connect";
import { QueryExprSchema, QueryModifiersSchema, QueryConstraintsSchema, CueSchema, type QueryExpr, type QueryModifiers, type Cue } from "@nous-wave/protocol/nous/wave/v1alpha1/types_pb.js";
import type { Ref } from "../domain.js";
import { canonical, parse } from "./parser.js";
import type { Atom, Directive, Expression, Locator } from "./syntax.js";

export type IdentityResolver = (kind: string, locator: Locator) => Promise<{ canonical: Ref; lexicalRef: string }>;
const selectorKinds = { e: "entity", tag: "tag", anchor: "anchor", r: "resource", ref: "", object: "external_object" };
function invalid(message: string): never { throw new ConnectError(message, Code.InvalidArgument); }

export async function compileNousQL(source: string, resolve: IdentityResolver, now = new Date()) {
  const syntax = parse(source);
  const sourceCanonical = canonical(syntax);
  async function bindAtom(atom: Atom): Promise<Cue[]> {
    if (atom.kind === "universe") return [];
    if (atom.kind === "text" || atom.kind === "concept") return [create(CueSchema, { cue: { case: atom.kind === "text" ? "text" : "concept", value: atom.text } })];
    const bound = await Promise.all(atom.locators.map(async locator => {
      if (atom.selector === "object") return { canonical: { kind: "external_object", value: locator.value }, lexicalRef: "" };
      return resolve(selectorKinds[atom.selector], locator);
    }));
    const identities = bound.map(b => `${b.canonical.kind}:${b.canonical.value}`);
    if (new Set(identities).size !== identities.length) invalid("DUPLICATE_ENTITY_PARTICIPANT");
    if (atom.selector === "e") bound.sort((a,b) => a.lexicalRef.localeCompare(b.lexicalRef));
    if (atom.selector !== "object") atom.locators = bound.map(b => ({ kind: "lexical", value: b.lexicalRef }));
    return bound.map(b => create(CueSchema, { cue: { case: "reference", value: b.canonical } }));
  }
  async function lower(node: Expression): Promise<QueryExpr> {
    const modifiers = create(QueryModifiersSchema, { constraints: {} });
    const seen = new Set<string>();
    for (const directive of node.directives) {
      if (seen.has(directive.name)) invalid(`Duplicate directive $${directive.name}`);
      seen.add(directive.name);
      applyDirective(modifiers, directive, now);
    }
    for (const preference of node.preferences) {
      if (preference.operand.kind === "key") {
        if (preference.operand.value !== "recent") invalid("Unknown soft preference key");
        modifiers.preferences.push({ $typeName: "nous.wave.v1alpha1.Preference", negative: preference.negative, key: "recent" });
      } else {
        const cues = await bindAtom(preference.operand);
        for (const cue of cues) modifiers.preferences.push({ $typeName: "nous.wave.v1alpha1.Preference", negative: preference.negative, key: "", cue });
      }
    }
    return create(QueryExprSchema, { operation: node.operation, cues: node.atom ? await bindAtom(node.atom) : [], children: await Promise.all(node.children.map(lower)), modifiers });
  }
  const expression = await lower(syntax);
  expression.modifiers ??= create(QueryModifiersSchema);
  expression.modifiers.limit ??= 12;
  return { expression, sourceCanonical, boundCanonical: canonical(syntax) };
}
function applyDirective(m: QueryModifiers, d: Directive, now: Date) {
  const c = m.constraints ??= create(QueryConstraintsSchema);
  const strings = () => { if (Object.keys(d.named).length || !d.positional.length || d.positional.some(v => typeof v !== "string")) invalid(`Invalid arguments for $${d.name}`); return d.positional as string[]; };
  const one = () => { const v = strings(); if (v.length !== 1) invalid(`$${d.name} expects one argument`); return v[0]!; };
  const noArguments = () => { if (d.positional.length || Object.keys(d.named).length) invalid(`$${d.name} takes no arguments`); };
  switch (d.name) {
    case "memory": case "evidence": case "resource": noArguments();m.domains.push(d.name);break;
    case "persona": case "relation": case "rel": throw new ConnectError(`${d.name} cognition is unavailable`, Code.FailedPrecondition);
    case "effort": { const value=one();if(!["light","normal","deep","maximum"].includes(value))invalid("Invalid effort");m.effort=value;break; }
    case "limit": { const value=d.positional[0];if(d.positional.length!==1||Object.keys(d.named).length||typeof value!=="number"||!Number.isInteger(value)||value<1||value>2048)invalid("Invalid limit");m.limit=value;break; }
    case "source": c.sourceClassesInclude=strings();break;
    case "modality": c.modalities=strings();break;
    case "memoryClass": c.memoryClassesInclude=strings();break;
    case "evidenceClass": c.evidenceClasses=strings();break;
    case "authority": c.authority=one();break;
    case "current": { const value=one();if(!["none","prefer","required","historical_or_stale"].includes(value))invalid("Invalid current-authority requirement");m.currentAuthority=value;break; }
    case "diagnostics": {const value=one();if(!["none","summary","full"].includes(value))invalid("Invalid diagnostics level");m.diagnostics=value;break;}
    case "explore": noArguments();m.exploration="bounded_associative";break;
    case "materialize": noArguments();m.materialize=true;break;
    case "exclude": {
      if(d.positional.length||Object.keys(d.named).length!==1||typeof d.named.source!=="string")invalid("Use $exclude(source=...)");
      c.sourceClassesExclude.push(d.named.source);break;
    }
    case "time": {
      const axis=d.positional[0];
      if(d.positional.length!==1||!["occurred","observed","valid"].includes(String(axis)))invalid("Time axis must be occurred, observed or valid");
      const keys=Object.keys(d.named);
      if(!keys.length||keys.some(k=>!["within","from","to","at"].includes(k)))invalid("Invalid time arguments");
      let start: Date|undefined;let end: Date|undefined;
      if(d.named.within!==undefined){
        if(keys.length!==1)invalid("within cannot be combined with other time arguments");
        const match=/^(\d+)(ms|s|m|h|d|w)$/.exec(String(d.named.within));if(!match)invalid("Invalid duration");
        const scale:Record<string,number>={ms:1,s:1000,m:60_000,h:3_600_000,d:86_400_000,w:604_800_000};
        const duration=Number(match[1])*scale[match[2]!]!;if(!Number.isSafeInteger(duration)||duration<=0)invalid("Invalid duration");
        end=now;start=new Date(now.getTime()-duration);
      } else if(d.named.at!==undefined){if(keys.length!==1)invalid("at cannot be combined with other time arguments");start=date(d.named.at);end=start;}
      else {start=d.named.from===undefined?undefined:date(d.named.from);end=d.named.to===undefined?undefined:date(d.named.to);}
      if(start&&end&&start>end)invalid("Time interval is reversed");
      c[axis as "occurred"|"observed"|"valid"]={$typeName:"nous.wave.v1alpha1.TimeInterval",start:start?timestamp(start):undefined,end:end?timestamp(end):undefined};break;
    }
    default: invalid(`Unknown directive $${d.name}`);
  }
}
function date(value:string|number) {if(typeof value!=="string"||!/^\d{4}-\d\d-\d\dT.*(?:Z|[+-]\d\d:\d\d)$/.test(value))invalid("Time requires an explicit ISO-8601 offset");const d=new Date(value);if(!Number.isFinite(d.getTime()))invalid("Invalid date");return d;}
function timestamp(value:Date){if(!Number.isFinite(value.getTime()))invalid("Invalid timestamp");return {$typeName:"google.protobuf.Timestamp" as const,seconds:BigInt(Math.floor(value.getTime()/1000)),nanos:((value.getTime()%1000+1000)%1000)*1_000_000};}
