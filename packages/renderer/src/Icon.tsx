import { createElement, type ComponentProps, type FunctionComponent } from "react";

export type IconNode = [tag: string, attrs: Record<string, string>][];

export type IconProps = Omit<ComponentProps<"svg">, "strokeWidth"> & {
  size?: number | string;
  strokeWidth?: number | string;
};

export type IconComponent = FunctionComponent<IconProps>;

export function IconSvg({ node, size = 24, strokeWidth = 2, ...rest }: IconProps & { node: IconNode }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden={rest["aria-label"] || rest["aria-labelledby"] ? undefined : true}
      {...rest}
    >
      {node.map(([tag, attrs], index) => createElement(tag, { key: index, ...attrs }))}
    </svg>
  );
}

export function createIcon(node: IconNode): IconComponent {
  return function Icon(props: IconProps) {
    return <IconSvg node={node} {...props} />;
  };
}

export function createIconSet<Name extends string>(nodes: Record<Name, unknown[][]>): Record<Name, IconComponent> {
  return Object.fromEntries(Object.entries<unknown[][]>(nodes).map(([name, node]) => [name, createIcon(node as IconNode)])) as Record<Name, IconComponent>;
}
