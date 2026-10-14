import { createContext, useContext, type CSSProperties, type ReactNode } from "react";
import type { Block, IconElement, ListItem, SourceRange, Theme } from "@ribaltalang/compiler";
import { IconSvg, type IconNode } from "./Icon";
import { useResolveSource } from "./assets";
import { Chart } from "./Chart";
import { layoutCss, visualCss } from "./css";
import { evaluate, formatScalar, holds } from "./expr";
import { Inlines } from "./Inlines";
import { behaviorCss, enterClass, InteractionContext, interactiveProps, useHover, useInteraction, withVariants } from "./interaction";
import { revealCss, useStep } from "./step";

export const ThemeContext = createContext<Theme | null>(null);

export const IconsContext = createContext<Record<string, IconElement[]>>({});

function iconNode(elements: IconElement[]): IconNode {
  return elements.map(({ tag, attrs }) => [tag, attrs]);
}

export function useTheme() {
  const theme = useContext(ThemeContext);
  if (!theme) throw new Error("ThemeContext missing");
  return theme;
}

function sourceAttr(source: SourceRange | undefined) {
  return source ? `${source.line}:${source.spans.map((span) => span.join("-")).join(",")}` : undefined;
}

export function Blocks({ blocks }: { blocks: Block[] }) {
  return (
    <>
      {blocks.map((block, index) => (block.type === "each" ? <EachView key={index} block={block} /> : <BlockView key={index} block={block} />))}
    </>
  );
}

type EachBlock = Extract<Block, { type: "each" }>;
type Cell = (child: Block, index: number) => ReactNode;

function EachView({ block, cell }: { block: EachBlock; cell?: Cell }) {
  const interaction = useInteraction();
  const list = evaluate(block.list, interaction.scope);
  if (!Array.isArray(list)) return null;
  return (
    <>
      {list.map((item, position) => (
        <InteractionContext.Provider
          key={position}
          value={{ ...interaction, scope: { ...interaction.scope, items: { ...interaction.scope.items, [block.item]: item, ...(block.index ? { [block.index]: position + 1 } : {}) } } }}
        >
          {cell ? <Cells blocks={block.children} cell={cell} /> : <Blocks blocks={block.children} />}
        </InteractionContext.Provider>
      ))}
    </>
  );
}

function Cells({ blocks, cell }: { blocks: Block[]; cell: Cell }) {
  const { scope } = useInteraction();
  return <>{blocks.map((child, index) => (child.type === "each" ? <EachView key={index} block={child} cell={cell} /> : holds(child.when, scope) ? cell(child, index) : null))}</>;
}

function ListItemView({ item }: { item: ListItem }) {
  const step = useStep();
  const theme = useTheme();
  const resolve = useResolveSource();
  const { scope, run } = useInteraction();
  const hover = useHover(item.variants);
  if (!holds(item.when, scope)) return null;
  const itemStyle = withVariants(item.style, item.variants, { ...scope, hovered: hover.hovered });
  const itemClickable = interactiveProps(item.click, run);
  const itemClick = itemClickable.onClick;
  const itemReveal = revealCss(item.step, step, theme.motion, itemStyle.enter);
  return (
    <li
      data-line={item.line}
      className={enterClass(itemStyle.enter, item.when !== undefined).trim() || undefined}
      style={{ ...visualCss(itemStyle, resolve), ...itemReveal, ...behaviorCss(item.variants, Boolean(itemClick), itemReveal, theme.motion) }}
      {...itemClickable}
      {...hover.handlers}
    >
      {item.source ? (
        <span data-line={item.line} data-editable="item" data-source={sourceAttr(item.source)} className="ribalta-item-text">
          <Inlines nodes={item.content} />
        </span>
      ) : (
        <Inlines nodes={item.content} />
      )}
      <Blocks blocks={item.children} />
    </li>
  );
}

function BlockView({ block }: { block: Block }) {
  const step = useStep();
  const theme = useTheme();
  const resolve = useResolveSource();
  const documentIcons = useContext(IconsContext);
  const { scope, run } = useInteraction();
  const hover = useHover(block.variants);
  if (!holds(block.when, scope)) return null;
  const style = withVariants(block.style, block.variants, { ...scope, hovered: hover.hovered });
  const clickable = interactiveProps(block.click, run);
  const onClick = clickable.onClick;
  const reveal = revealCss(block.step, step, theme.motion, style.enter);
  const base: CSSProperties = { ...visualCss(style, resolve), ...reveal, ...behaviorCss(block.variants, Boolean(onClick), reveal, theme.motion) };
  const enter = enterClass(style.enter, block.when !== undefined);
  const interactive = { ...clickable, ...hover.handlers };
  const line = block.line;
  const range = block.endLine ? `${line}-${block.endLine}` : undefined;

  switch (block.type) {
    case "heading": {
      const defaults = visualCss(theme.headings[block.level - 1] ?? {}, resolve);
      const Tag = `h${Math.min(block.level, 6)}` as "h1";
      return (
        <Tag data-line={line} data-block={range} data-editable={block.source ? "heading" : undefined} data-source={sourceAttr(block.source)} className={`ribalta-heading${enter}`} style={{ ...defaults, ...base }} {...interactive}>
          <Inlines nodes={block.content} />
        </Tag>
      );
    }
    case "paragraph":
      return (
        <p data-line={line} data-block={range} data-editable={block.source ? "paragraph" : undefined} data-source={sourceAttr(block.source)} className={`ribalta-paragraph${enter}`} style={base} {...interactive}>
          <Inlines nodes={block.content} />
        </p>
      );
    case "list": {
      const Tag = block.ordered ? "ol" : "ul";
      return (
        <Tag data-line={line} data-block={range} className={`ribalta-list${enter}`} start={block.ordered ? block.start : undefined} style={base} {...interactive}>
          {block.items.map((item, index) => (
            <ListItemView key={index} item={item} />
          ))}
        </Tag>
      );
    }
    case "quote":
      return (
        <blockquote data-line={line} data-block={range} className={`ribalta-quote${enter}`} style={base} {...interactive}>
          <Blocks blocks={block.children} />
        </blockquote>
      );
    case "code": {
      const { syntax } = theme.elements.code;
      return (
        <pre data-line={line} data-block={range} className="ribalta-code" style={base}>
          {block.lines.map((tokens, index) => (
            <div key={index}>
              {tokens.length
                ? tokens.map((token, key) => (
                    <span key={key} style={{ color: syntax[token.kind]?.color, fontStyle: syntax[token.kind]?.italic ? "italic" : undefined }}>
                      {token.text}
                    </span>
                  ))
                : "\n"}
            </div>
          ))}
        </pre>
      );
    }
    case "table":
      return (
        <table data-line={line} data-block={range} className={`ribalta-table${enter}`} style={base} {...interactive}>
          <thead>
            <tr>
              {block.header.map((cell, index) => (
                <th key={index} style={{ textAlign: block.aligns[index] ?? "left" }}>
                  <Inlines nodes={cell} />
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {block.rows.map((row, rowIndex) => (
              <tr key={rowIndex}>
                {row.map((cell, index) => (
                  <td key={index} style={{ textAlign: block.aligns[index] ?? "left" }}>
                    <Inlines nodes={cell} />
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      );
    case "image":
      return (
        <img
          data-line={line}
          data-block={range}
          className={`ribalta-image${enter}`}
          src={resolve(block.src)}
          alt={block.alt}
          referrerPolicy="no-referrer"
          draggable={false}
          style={{ objectFit: block.fit, ...base }}
          {...interactive}
        />
      );
    case "layout":
      return (
        <div
          data-line={line}
          data-block={range}
          className={`ribalta-layout ribalta-${block.layout}${enter}`}
          style={{ ...layoutCss(block.layout, style, block.cols, block.wrap, theme.layout), ...base }}
          {...interactive}
        >
          <Cells
            blocks={block.children}
            cell={(child, index) => (
              <div
                key={index}
                className="ribalta-cell"
                style={block.layout === "columns" ? { flex: `${(child === block.children[index] ? block.widths?.[index] : undefined) ?? 1} 1 0`, minWidth: 0, display: "flex", flexDirection: "column" } : { display: "contents" }}
              >
                <BlockView block={child} />
              </div>
            )}
          />
        </div>
      );
    case "chart":
      return (
        <div data-line={line} data-block={range} className={`ribalta-chart${enter}`} style={{ width: "100%", ...base }} {...interactive}>
          <Chart block={block} />
        </div>
      );
    case "icon": {
      const elements = documentIcons[block.name];
      if (!elements) return null;
      return (
        <IconSvg
          node={iconNode(elements)}
          data-line={line}
          data-block={range}
          className={`ribalta-icon${enter}`}
          size={style.size ?? theme.elements.icon.size}
          strokeWidth={theme.elements.icon.stroke}
          style={{ ...base, flexShrink: 0 }}
          aria-hidden
          {...interactive}
        />
      );
    }
    case "spacer":
      return <div data-line={line} data-block={range} style={{ flex: `0 0 ${block.size}px` }} />;
    case "divider":
      return <hr data-line={line} data-block={range} className={`ribalta-divider${enter}`} style={base} />;
    case "sound":
      return null;
    case "input": {
      const current = scope.state[block.name];
      const write = (raw: string) => run?.([{ type: "set", name: block.name, value: { op: "lit", value: block.numeric ? Number(raw) || 0 : raw } }]);
      return (
        <input
          data-line={line}
          data-block={range}
          data-interactive=""
          data-submit={block.onSubmit?.length ? "" : undefined}
          className={`ribalta-input${enter}`}
          type={block.numeric ? "number" : "text"}
          value={current === undefined ? "" : formatScalar(current)}
          placeholder={block.placeholder}
          aria-label={block.placeholder ?? block.name}
          maxLength={200}
          disabled={!run}
          onChange={(event) => write(event.target.value)}
          onKeyDown={(event) => {
            if (event.key !== "Enter" || !block.onSubmit?.length) return;
            event.preventDefault();
            run?.(block.onSubmit);
          }}
          onClick={(event) => event.stopPropagation()}
          onPointerDown={(event) => event.stopPropagation()}
          style={{ ...inputCss(theme), ...base }}
        />
      );
    }
  }
}

function inputCss(theme: Theme): CSSProperties {
  return {
    font: "inherit",
    color: "inherit",
    width: "min(100%, 18em)",
    padding: "0.3em 0.6em",
    border: `3px solid ${theme.colors["line"] ?? "currentColor"}`,
    borderRadius: 16,
    background: theme.colors["surface"] ?? "transparent",
  };
}
