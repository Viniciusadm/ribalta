import { useEffect, useRef } from "react";
import type { Block, Document, Page } from "@ribaltalang/compiler";
import { useResolveSource } from "@ribaltalang/renderer/assets";
import type { Session } from "./navigation";

type SoundBlock = Extract<Block, { type: "sound" }>;

function soundsOf(page: Page | undefined): SoundBlock[] {
  const found: SoundBlock[] = [];
  const visit = (blocks: Block[]) => {
    for (const block of blocks) {
      if (block.type === "sound") found.push(block);
      else if (block.type === "layout" || block.type === "quote") visit(block.children);
      else if (block.type === "list") for (const item of block.items) visit(item.children);
    }
  };
  if (page) visit([page.header, ...page.children, page.footer, page.overlay].filter((block): block is Block => block !== undefined));
  return found;
}

function start(audio: HTMLAudioElement) {
  try {
    audio.currentTime = 0;
    void audio.play()?.catch(() => undefined);
  } catch {
    return;
  }
}

function halt(audio: HTMLAudioElement) {
  try {
    audio.pause();
    audio.currentTime = 0;
  } catch {
    return;
  }
}

export function useSounds(doc: Document | null, session: Session | null, { enabled = true, fromStart = true } = {}) {
  const resolve = useResolveSource();
  const playing = useRef(new Map<string, HTMLAudioElement>());
  const seen = useRef<number | null>(fromStart ? 0 : null);
  const index = session?.index;
  const page = doc && index !== undefined ? doc.pages[index] : undefined;

  useEffect(() => {
    if (!enabled || !page) return;
    const audios = new Map<string, HTMLAudioElement>();
    const unnamed: HTMLAudioElement[] = [];
    for (const sound of soundsOf(page)) {
      const audio = new Audio(resolve(sound.src));
      audio.loop = sound.loop === true;
      audio.volume = sound.volume ?? 1;
      if (sound.name) audios.set(sound.name, audio);
      else unnamed.push(audio);
      if (sound.autoplay) start(audio);
    }
    playing.current = audios;
    return () => {
      for (const audio of [...audios.values(), ...unnamed]) halt(audio);
      playing.current = new Map();
    };
  }, [enabled, page, resolve]);

  const cues = session?.cues;
  useEffect(() => {
    if (!cues) return;
    if (seen.current === null) {
      seen.current = cues[cues.length - 1]?.id ?? 0;
      return;
    }
    for (const cue of cues) {
      if (cue.id <= seen.current) continue;
      seen.current = cue.id;
      const audio = playing.current.get(cue.name);
      if (!enabled || !audio) continue;
      if (cue.type === "play") start(audio);
      else halt(audio);
    }
  }, [cues, enabled]);
}
