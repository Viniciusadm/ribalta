import { Fragment, type MouseEvent } from "react";
import type { Inline, Style } from "@ribaltalang/compiler";
import { htmlLang, useLocale } from "./locale";
import { sourceText, useResolveSource } from "./assets";
import { useTheme } from "./Blocks";
import { visualCss } from "./css";
import { evaluate, formatScalar, holds } from "./expr";
import { behaviorCss, clickHandler, interactiveProps, useHover, useInteraction, withVariants } from "./interaction";
import { useStep, revealCss } from "./step";

export function Inlines({ nodes }: { nodes: Inline[] }) {
  return (
    <>
      {nodes.map((node, index) => (
        <InlineNode key={index} node={node} />
      ))}
    </>
  );
}

function boxed(style: Style) {
  return Boolean(style.pad || style.bg || style.border || style.radius !== undefined);
}

function Today() {
  const locale = useLocale();
  return <Fragment>{new Date().toLocaleDateString(htmlLang[locale], { day: "numeric", month: "long", year: "numeric" })}</Fragment>;
}

function InlineNode({ node }: { node: Inline }) {
  const step = useStep();
  const resolve = useResolveSource();
  const { scope, run, scrollTo } = useInteraction();
  const hover = useHover(node.type === "span" ? node.variants : undefined);
  const { motion } = useTheme();
  switch (node.type) {
    case "text":
      return <Fragment>{node.value}</Fragment>;
    case "break":
      return <br />;
    case "field":
      return <Today />;
    case "expr":
      return <Fragment>{formatScalar(evaluate(node.expr, scope))}</Fragment>;
    case "strong":
      return (
        <strong>
          <Inlines nodes={node.children} />
        </strong>
      );
    case "emphasis":
      return (
        <em>
          <Inlines nodes={node.children} />
        </em>
      );
    case "strike":
      return (
        <s>
          <Inlines nodes={node.children} />
        </s>
      );
    case "code":
      return <code className="ribalta-inline-code">{node.value}</code>;
    case "link": {
      if (node.href.startsWith("#")) {
        const target = node.href.slice(1);
        const onClick =
          clickHandler([{ type: "go", target }], run) ??
          ((event: MouseEvent) => {
            event.preventDefault();
            scrollTo?.(target);
          });
        return (
          <a href={node.href} className="ribalta-link" onClick={onClick} data-interactive="">
            <Inlines nodes={node.children} />
          </a>
        );
      }
      return (
        <a href={node.href} target="_blank" rel="noopener noreferrer nofollow" className="ribalta-link" onClick={(event) => event.stopPropagation()}>
          <Inlines nodes={node.children} />
        </a>
      );
    }
    case "span": {
      if (!holds(node.when, scope)) return null;
      const style = withVariants(node.style, node.variants, { ...scope, hovered: hover.hovered });
      const clickable = interactiveProps(node.click, run);
      const onClick = clickable.onClick;
      const reveal = revealCss(node.step, step, motion, style.enter);
      return (
        <span
          data-attrs={node.sourceAttrs ?? ""}
          style={{ ...visualCss(style, resolve), ...(boxed(style) && { display: "inline-block" }), ...reveal, ...behaviorCss(node.variants, Boolean(onClick), reveal, motion) }}
          {...clickable}
          {...hover.handlers}
        >
          <Inlines nodes={node.children} />
        </span>
      );
    }
    case "image":
      return (
        <img
          src={resolve(node.src)}
          alt={node.alt}
          data-src={sourceText(node.src)}
          data-attrs={node.sourceAttrs ?? ""}
          referrerPolicy="no-referrer"
          draggable={false}
          style={{ height: "1em", verticalAlign: "middle", ...visualCss(node.style, resolve) }}
        />
      );
  }
}
