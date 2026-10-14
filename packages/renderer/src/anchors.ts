export function findAnchor(root: ParentNode, id: string): Element | null {
  return root.querySelector(`[data-anchor="${CSS.escape(id)}"]`);
}

export function scrollToAnchor(root: ParentNode, id: string): boolean {
  const element = findAnchor(root, id);
  if (!element) return false;
  element.scrollIntoView({ block: "start", behavior: "smooth" });
  return true;
}
