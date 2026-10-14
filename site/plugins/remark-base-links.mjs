function visit(node, base) {
  if (typeof node.url === "string" && node.url.startsWith("/") && !node.url.startsWith(`${base}/`)) node.url = `${base}${node.url}`;
  for (const child of node.children ?? []) visit(child, base);
}

export function remarkBaseLinks({ base }) {
  return (tree) => visit(tree, base.replace(/\/$/, ""));
}
