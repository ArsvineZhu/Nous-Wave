import { expect, it } from "vitest";
import { canonical, parse } from "../src/nousql/parser.js";
import { compileNousQL } from "../src/nousql/compiler.js";

const resolve = async (kind:string,locator:{value:string}) => ({canonical:{kind,value:`entity:${locator.value}`},lexicalRef:locator.value==="Alice"?"ent:amber-lotus-cello-river":"ent:quiet-piano-mint-cloud"});
it("preserves Boolean scope and simplifies soft preference grouping", () => {
  const syntax=parse('(@e("Alice") $limit(5)) || (@e("Bob") $limit(2))');
  expect(syntax.directives).toHaveLength(0);
  expect(syntax.children.map(c=>c.directives[0]?.positional[0])).toEqual([5,2]);
  expect(canonical(parse('@e("Alice") +("school") -"noise"'))).toBe('@e("Alice") +"school" -"noise"');
  expect(()=>parse('~"school"')).toThrow();
  expect(()=>parse('"a" "b"')).toThrow();
  expect(()=>parse('("a"')).toThrow();
});
it("binds names exactly, sorts joint participants, and rejects duplicate identity",async()=>{
  const result=await compileNousQL('@e("Bob","Alice") $memory +"school"',resolve);
  expect(result.boundCanonical).toContain('@e(ent:amber-lotus-cello-river,ent:quiet-piano-mint-cloud)');
  expect(result.expression.cues).toHaveLength(2);
  expect(result.expression.modifiers?.preferences[0]?.negative).toBe(false);
  await expect(compileNousQL('@e("Alice","Alice")',resolve)).rejects.toThrow("DUPLICATE_ENTITY_PARTICIPANT");
});
it("keeps independent time axes and rejects ambiguous or duplicate modifiers",async()=>{
  const now=new Date('2026-09-17T00:00:00Z');
  const result=await compileNousQL('"study" $time(observed,within=30d) $effort(deep)',resolve,now);
  expect(result.expression.modifiers?.constraints?.observed?.end?.seconds).toBe(BigInt(now.getTime()/1000));
  expect(result.expression.modifiers?.constraints?.occurred).toBeUndefined();
  await expect(compileNousQL('"study" $limit(3) $limit(4)',resolve)).rejects.toThrow("Duplicate");
  await expect(compileNousQL('"study" $time(valid,at="2026-09-17")',resolve)).rejects.toThrow("offset");
  await expect(compileNousQL('"study" $persona',resolve)).rejects.toThrow("unavailable");
});
