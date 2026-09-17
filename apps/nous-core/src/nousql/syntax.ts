export interface Locator { kind: "name" | "lexical"; value: string }
export interface Selector { kind: "selector"; selector: "e" | "tag" | "anchor" | "r" | "object" | "ref"; locators: Locator[] }
export type Atom = { kind: "text"; text: string } | { kind: "concept"; text: string } | Selector | { kind: "universe" };
export type Argument = string | number;
export interface Directive { name: string; positional: Argument[]; named: Record<string, Argument> }
export interface Preference { negative: boolean; operand: Atom | { kind: "key"; value: string } }
export interface Expression {
  operation: "atom" | "all" | "any";
  atom?: Atom;
  children: Expression[];
  directives: Directive[];
  preferences: Preference[];
}
export const expression = (atom: Atom): Expression => ({ operation: "atom", atom, children: [], directives: [], preferences: [] });
