import { createContext, useContext, useState, type CSSProperties, type FocusEvent, type KeyboardEvent, type MouseEvent, type PointerEvent } from "react";
import type { Action, Motion, Style, Variant } from "@ribaltalang/compiler";
import { holds, type Scope } from "./expr";
import { timingCss } from "./motion";

export interface Interaction {
  scope: Scope;
  run: ((actions: Action[]) => void) | null;
  scrollTo?: (id: string) => boolean;
}

export const InteractionContext = createContext<Interaction>({ scope: { state: {}, step: Number.POSITIVE_INFINITY }, run: null });

export function useInteraction() {
  return useContext(InteractionContext);
}

export function useHover(variants: Variant[] | undefined) {
  const [hovered, setHovered] = useState(false);
  if (!variants?.some((variant) => variant.when.op === "hover")) return { hovered: false, handlers: {} };
  return {
    hovered,
    handlers: {
      onPointerEnter: (event: PointerEvent) => {
        if (event.pointerType !== "touch") setHovered(true);
      },
      onPointerLeave: () => setHovered(false),
      onFocus: (event: FocusEvent) => setHovered(focusVisible(event.target)),
      onBlur: () => setHovered(false),
    },
  };
}

export function withVariants(style: Style, variants: Variant[] | undefined, scope: Scope): Style {
  if (!variants?.length) return style;
  return variants.reduce((merged, variant) => (holds(variant.when, scope) ? { ...merged, ...variant.style } : merged), style);
}

const variantProperties = ["opacity", "color", "background-color", "box-shadow", "border-color", "transform"];

export function behaviorCss(variants: Variant[] | undefined, clickable: boolean, base: CSSProperties, motion: Motion): CSSProperties {
  const css: CSSProperties = {};
  const variantTransition = timingCss(variantProperties, motion.variant);
  if (variants?.length) css.transition = base.transition ? `${base.transition}, ${variantTransition}` : variantTransition;
  if (clickable) css.cursor = "pointer";
  return css;
}

export function clickHandler(actions: Action[] | undefined, run: Interaction["run"]) {
  if (!actions?.length || !run) return undefined;
  return (event: MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    run(actions);
  };
}

function focusVisible(element: Element) {
  try {
    return element.matches(":focus-visible");
  } catch {
    return false;
  }
}

export function interactiveProps(actions: Action[] | undefined, run: Interaction["run"]) {
  const onClick = clickHandler(actions, run);
  if (!onClick || !actions || !run) return {};
  return {
    onClick,
    "data-interactive": "",
    role: "button",
    tabIndex: 0,
    onKeyDown: (event: KeyboardEvent) => {
      if (event.key !== "Enter" && event.key !== " ") return;
      event.preventDefault();
      event.stopPropagation();
      run(actions);
    },
  };
}

export function enterClass(enter: Style["enter"], conditional: boolean) {
  return conditional && enter && enter !== "none" ? ` ribalta-enter-${enter}` : "";
}
