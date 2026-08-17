export type EffortLevel = 'low' | 'medium' | 'high';

export const EFFORT_OPTIONS: Record<
  EffortLevel,
  { label: string; hint: string }
> = {
  low: { label: 'Low', hint: 'Fast answers, less depth' },
  medium: { label: 'Medium', hint: 'Balanced speed and quality' },
  high: { label: 'High', hint: 'Deeper reasoning, slower' },
};

export function effortLabelFor(level: EffortLevel): string {
  return EFFORT_OPTIONS[level].label;
}

export function effortToCompressionRatio(effort: EffortLevel): number {
  if (effort === 'low') return 0.5;
  return 0.78;
}

export function effortToMaxOutputTokens(effort: EffortLevel): number {
  if (effort === 'high') return 10_000;
  if (effort === 'low') return 3_000;
  return 5_000;
}
