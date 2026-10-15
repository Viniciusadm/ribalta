export const SHRINK_VAR = "--ribalta-shrink";

const SHRINK_SELECTOR = `[style*="${SHRINK_VAR}"]`;
const TOLERANCE = 1;

export function shrinkFontSize(size: number | undefined, min: number): string {
  if (size === undefined) return `max(${min}px, calc(1em * var(${SHRINK_VAR}, 1)))`;
  return `max(${Math.min(min, size)}px, calc(${size}px * var(${SHRINK_VAR}, 1)))`;
}

export function searchShrink(apply: (factor: number) => void, overflows: () => boolean, steps = 10): number {
  apply(1);
  if (!overflows()) return 1;
  apply(0);
  if (overflows()) return 0;
  let fits = 0;
  let tooBig = 1;
  for (let step = 0; step < steps; step += 1) {
    const middle = (fits + tooBig) / 2;
    apply(middle);
    if (overflows()) tooBig = middle;
    else fits = middle;
  }
  apply(fits);
  return fits;
}

function spills(element: Element) {
  return element.scrollHeight > element.clientHeight + TOLERANCE || element.scrollWidth > element.clientWidth + TOLERANCE;
}

function overflowing(page: HTMLElement, targets: HTMLElement[]) {
  if (spills(page)) return true;
  for (const target of targets) {
    if (target.clientWidth > 0 && target.scrollWidth > target.clientWidth + TOLERANCE) return true;
    for (let parent = target.parentElement; parent && parent !== page; parent = parent.parentElement) {
      if (spills(parent)) return true;
    }
  }
  return false;
}

export function fitShrink(page: HTMLElement): number {
  const targets = Array.from(page.querySelectorAll<HTMLElement>(SHRINK_SELECTOR));
  if (targets.length === 0) {
    page.style.removeProperty(SHRINK_VAR);
    return 1;
  }
  return searchShrink(
    (factor) => page.style.setProperty(SHRINK_VAR, String(factor)),
    () => overflowing(page, targets),
  );
}
