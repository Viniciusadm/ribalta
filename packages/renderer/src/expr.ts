import type { Expr, Field, Row, Scalar } from "@ribaltalang/compiler";

export interface Scope {
  state: Record<string, Scalar>;
  items?: Record<string, Field | Row>;
  step: number;
  draw?: () => number;
  hovered?: boolean;
  elapsed?: number;
}

export function nextRandom(seed: number): { value: number; seed: number } {
  const next = (seed + 0x6d2b79f5) >>> 0;
  let mixed = Math.imul(next ^ (next >>> 15), next | 1);
  mixed ^= mixed + Math.imul(mixed ^ (mixed >>> 7), mixed | 61);
  return { value: ((mixed ^ (mixed >>> 14)) >>> 0) / 4294967296, seed: next };
}

export function formatScalar(value: Scalar): string {
  if (Array.isArray(value)) return value.join(", ");
  if (typeof value !== "number") return String(value);
  if (value === 0) return "0";
  if (Number.isInteger(value) && Math.abs(value) < 1e15) return value.toFixed(0);
  return String(value);
}

function roundTo(value: number, digits: number) {
  const factor = 10 ** Math.trunc(Math.min(Math.max(digits, 0), 10));
  const scaled = value * factor;
  return (Math.sign(scaled) * Math.round(Math.abs(scaled))) / factor;
}

function number(value: Scalar) {
  if (Array.isArray(value)) return 0;
  return typeof value === "number" ? value : Number(value) || 0;
}

function length(value: Scalar) {
  if (Array.isArray(value)) return value.length;
  return typeof value === "string" ? [...value].length : 0;
}

export function evaluate(expr: Expr, scope: Scope): Scalar {
  switch (expr.op) {
    case "lit":
      return expr.value;
    case "state":
      return scope.state[expr.name] ?? false;
    case "item": {
      const item = scope.items?.[expr.name];
      if (item === undefined) return false;
      if (typeof item !== "object") return item;
      return expr.field === undefined ? false : (item[expr.field] ?? false);
    }
    case "step":
      return scope.step;
    case "hover":
      return scope.hovered === true;
    case "elapsed":
      return scope.elapsed ?? 0;
    case "not":
      return !evaluate(expr.arg, scope);
    case "neg":
      return -number(evaluate(expr.arg, scope));
    case "cond":
      return evaluate(expr.test, scope) === true ? evaluate(expr.then, scope) : evaluate(expr.otherwise, scope);
    case "call": {
      if (expr.name === "len") return length(evaluate(expr.args[0] ?? { op: "lit", value: "" }, scope));
      if (expr.name === "pick") {
        const items: Field[] = [];
        for (const arg of expr.args) {
          const value = evaluate(arg, scope);
          if (!Array.isArray(value)) items.push(value);
          else for (const item of value) if (typeof item !== "object") items.push(item);
        }
        if (!items.length || !scope.draw) return "";
        return items[Math.floor(scope.draw() * items.length)]!;
      }
      if (expr.name === "random") {
        const [a, b] = expr.args.map((arg) => number(evaluate(arg, scope)));
        const low = Math.ceil(Math.min(a ?? 0, b ?? 0));
        const high = Math.max(low, Math.floor(Math.max(a ?? 0, b ?? 0)));
        return scope.draw ? low + Math.floor(scope.draw() * (high - low + 1)) : low;
      }
      const args = expr.args.map((arg) => number(evaluate(arg, scope)));
      if (expr.name === "round") return roundTo(args[0] ?? 0, args[1] ?? 0);
      if (expr.name === "min") return Math.min(...args);
      if (expr.name === "max") return Math.max(...args);
      return 0;
    }
    case "binary": {
      if (expr.operator === "and") return evaluate(expr.left, scope) === true && evaluate(expr.right, scope) === true;
      if (expr.operator === "or") return evaluate(expr.left, scope) === true || evaluate(expr.right, scope) === true;
      const left = evaluate(expr.left, scope);
      const right = evaluate(expr.right, scope);
      switch (expr.operator) {
        case "in":
          return Array.isArray(right) && typeof left === "string" && right.some((item) => item === left);
        case "==":
          return left === right;
        case "!=":
          return left !== right;
        case "<":
          return left < right;
        case "<=":
          return left <= right;
        case ">":
          return left > right;
        case ">=":
          return left >= right;
        case "+":
          if (typeof left === "string" || typeof right === "string") return formatScalar(left) + formatScalar(right);
          return number(left) + number(right);
        case "-":
          return number(left) - number(right);
        case "*":
          return number(left) * number(right);
        case "/":
          return number(right) === 0 ? 0 : number(left) / number(right);
        case "%":
          return number(right) === 0 ? 0 : number(left) % number(right);
      }
    }
  }
}

export function holds(expr: Expr | undefined, scope: Scope) {
  return expr === undefined || evaluate(expr, scope) === true;
}
