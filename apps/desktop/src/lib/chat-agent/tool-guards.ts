const VISUAL_PROMPT_RE =
  /\b(chart|graph|diagram|illustrat|infographic|bar\s+chart|line\s+chart|pie\s+chart|flowchart|mind\s*map|timeline|axis|axes|scatter|plot|visuali[sz]ation|icon|logo|map\b|figure\b|render|photoreal|3d\b|sketch|drawing|image\s+of|picture\s+of|photo|screenshot|mockup|blueprint|schematic|histogram|heatmap|dashboard)\b/i;

function proseSentenceCount(text: string): number {
  return text.split(/[.!?]+/).filter((part) => part.trim().length > 24).length;
}

export function validateGenerateImageCall(
  imagePrompt: string,
): { ok: true } | { ok: false; message: string } {
  const prompt = imagePrompt.trim();
  if (!prompt) {
    return {
      ok: false,
      message:
        'Empty image prompt. If the user wanted text (paragraph, draft, rewrite), write it in your chat reply — do not call generate_image.',
    };
  }

  if (VISUAL_PROMPT_RE.test(prompt)) {
    return { ok: true };
  }

  const longProse = prompt.length > 180 && proseSentenceCount(prompt) >= 2;
  const essayLike = prompt.length > 320;
  if (longProse || essayLike) {
    return {
      ok: false,
      message:
        'WRONG_TOOL: This prompt reads like paragraph prose, not a visual image spec. Respond with the full text in your chat message instead. Use generate_image only when the user wants a chart, diagram, illustration, or picture.',
    };
  }

  return { ok: true };
}
