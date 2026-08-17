export interface EuryMarkProps {
  size?: number;
  className?: string;
}

export function EuryMark({ size = 16, className = "" }: EuryMarkProps) {
  return (
    <span
      className={`eury-mark ${className}`.trim()}
      style={{ width: size, height: size, fontSize: Math.round(size * 0.56) }}
      aria-hidden="true"
    >
      E
    </span>
  );
}
