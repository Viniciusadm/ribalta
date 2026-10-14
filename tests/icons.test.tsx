import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import bundledIcons from "../crates/ribalta/icons/icons.json";
import { icons } from "@ribaltalang/renderer/icons";

const svgTags = new Set(["path", "circle", "rect", "line", "polyline", "polygon", "ellipse"]);

afterEach(() => cleanup());

describe("icon data", () => {
  it("keeps bundled icons as plain svg primitives", () => {
    for (const [name, node] of Object.entries<unknown[][]>(bundledIcons)) {
      expect(node.length, name).toBeGreaterThan(0);
      for (const [tag, attrs] of node) {
        expect(svgTags.has(tag as string), `${name}: ${tag}`).toBe(true);
        expect(Object.values(attrs as object).every((value) => typeof value === "string"), name).toBe(true);
      }
    }
  });

  it("renders an icon as a decorative svg", () => {
    const Rocket = icons["rocket"]!;
    const { container } = render(<Rocket size={40} strokeWidth={1.75} />);
    const svg = container.querySelector("svg")!;
    expect(svg.getAttribute("width")).toBe("40");
    expect(svg.getAttribute("stroke-width")).toBe("1.75");
    expect(svg.getAttribute("aria-hidden")).toBe("true");
    expect(svg.querySelectorAll("path")).toHaveLength(4);
  });
});
