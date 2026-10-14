import { useMemo } from "react";
import type { Animation, Enter, Length, Motion, Pose, Timing } from "@ribaltalang/compiler";

function lengthText(length: Length | undefined) {
  if (!length) return "0";
  if (length.unit === "px") return `${length.value}px`;
  if (length.unit === "pct") return `${length.value}%`;
  return "0";
}

export function easingCss([x1, y1, x2, y2]: Timing["easing"]) {
  return `cubic-bezier(${x1}, ${y1}, ${x2}, ${y2})`;
}

export function timingCss(properties: string[], timing: Timing) {
  return properties.map((property) => `${property} ${timing.duration}ms ${easingCss(timing.easing)}`).join(", ");
}

export function poseTransform(pose: Pose) {
  const parts: string[] = [];
  if (pose.dx || pose.dy) parts.push(`translate(${lengthText(pose.dx)}, ${lengthText(pose.dy)})`);
  if (pose.scale !== undefined) parts.push(`scale(${pose.scale})`);
  return parts.join(" ") || "none";
}

export function enterPose(motion: Motion, enter: Enter) {
  return motion.enter[enter]?.from;
}

function animationRules(prefix: string, animations: Partial<Record<string, Animation>>) {
  return Object.entries(animations)
    .flatMap(([name, animation]) => {
      if (!animation) return [];
      const id = `${prefix}-${name}`;
      const { from } = animation;
      return [
        `@keyframes ${id} { from { opacity: ${from.opacity}; transform: ${poseTransform(from)}; } }`,
        `.${id} { animation: ${id} ${animation.duration}ms ${easingCss(animation.easing)} both; }`,
      ];
    })
    .join("\n");
}

export function motionStylesheet(motion: Motion) {
  return [
    animationRules("ribalta-enter", motion.enter),
    animationRules("ribalta-transition", motion.transitions),
    `@media (prefers-reduced-motion: reduce) { [class*="ribalta-enter-"], [class*="ribalta-transition-"] { animation: none; } }`,
  ].join("\n");
}

function hash(text: string) {
  let value = 5381;
  for (let index = 0; index < text.length; index++) value = ((value << 5) + value + text.charCodeAt(index)) | 0;
  return (value >>> 0).toString(36);
}

export function MotionStyles({ motion }: { motion: Motion }) {
  const css = useMemo(() => motionStylesheet(motion), [motion]);
  return (
    <style href={`ribalta-motion-${hash(css)}`} precedence="ribalta">
      {css}
    </style>
  );
}
