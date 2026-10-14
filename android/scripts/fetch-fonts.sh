#!/usr/bin/env bash
set -euo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
out="$here/ribalta/src/main/assets/fonts"
base="https://raw.githubusercontent.com/google/fonts/main/ofl"
mkdir -p "$out"

fetch() {
  local path="$1" name="$2"
  [[ -s "$out/$name" ]] || curl -fsSL "$base/$path" -o "$out/$name"
}

fetch "dmsans/DMSans%5Bopsz,wght%5D.ttf" "DMSans-Variable.ttf"
fetch "fraunces/Fraunces%5BSOFT,WONK,opsz,wght%5D.ttf" "Fraunces-Variable.ttf"
fetch "inter/Inter%5Bopsz,wght%5D.ttf" "Inter-Variable.ttf"
fetch "intertight/InterTight%5Bwght%5D.ttf" "InterTight-Variable.ttf"
fetch "jetbrainsmono/JetBrainsMono%5Bwght%5D.ttf" "JetBrainsMono-Variable.ttf"
fetch "nunito/Nunito%5Bwght%5D.ttf" "Nunito-Variable.ttf"
fetch "playfairdisplay/PlayfairDisplay%5Bwght%5D.ttf" "PlayfairDisplay-Variable.ttf"
fetch "sourcesans3/SourceSans3%5Bwght%5D.ttf" "SourceSans3-Variable.ttf"
fetch "spacegrotesk/SpaceGrotesk%5Bwght%5D.ttf" "SpaceGrotesk-Variable.ttf"
fetch "archivoblack/ArchivoBlack-Regular.ttf" "ArchivoBlack-Regular.ttf"
for weight in Thin ExtraLight Light Regular Medium SemiBold Bold ExtraBold Black; do
  fetch "barlow/Barlow-$weight.ttf" "Barlow-$weight.ttf"
done
mkdir -p "$out/licenses"
for family in dmsans fraunces inter intertight jetbrainsmono nunito playfairdisplay sourcesans3 spacegrotesk archivoblack barlow; do
  [[ -s "$out/licenses/$family.txt" ]] || curl -fsSL "$base/$family/OFL.txt" -o "$out/licenses/$family.txt"
done
