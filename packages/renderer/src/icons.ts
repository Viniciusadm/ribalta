import { createIconSet, type IconComponent } from "./Icon";
import bundledIcons from "../../../crates/ribalta/icons/icons.json";

export const icons: Record<string, IconComponent> = createIconSet(bundledIcons);
