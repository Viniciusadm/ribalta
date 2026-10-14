const variableFamilies = [
  "DM Sans",
  "Fraunces",
  "Inter",
  "Inter Tight",
  "JetBrains Mono",
  "Nunito",
  "Playfair Display",
  "Source Sans 3",
  "Space Grotesk",
];
const staticFamilies = ["Archivo Black", "Barlow"];

const localFaces = new Map<string, string>([
  ...variableFamilies.map((family) => [family, `${family} Variable`] as const),
  ...staticFamilies.map((family) => [family, family] as const),
]);

export function localFontFace(family: string) {
  return localFaces.get(family);
}
