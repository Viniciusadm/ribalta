import { createContext, useContext, type CSSProperties } from "react";
import type { Enter, Motion } from "@ribaltalang/compiler";
import { enterPose, poseTransform, timingCss } from "./motion";

export const StepContext = createContext(Number.POSITIVE_INFINITY);

export function useStep() {
  return useContext(StepContext);
}

export function revealCss(step: number | undefined, current: number, motion: Motion, enter: Enter = "up"): CSSProperties {
  if (step === undefined) return {};
  if (enter === "none") return current >= step ? {} : { visibility: "hidden" };
  const transition = timingCss(["opacity", "transform", "visibility"], motion.reveal);
  if (current >= step) return { transition };
  const pose = enterPose(motion, enter);
  return { opacity: pose?.opacity ?? 0, visibility: "hidden", transform: pose ? poseTransform(pose) : "none", transition };
}
