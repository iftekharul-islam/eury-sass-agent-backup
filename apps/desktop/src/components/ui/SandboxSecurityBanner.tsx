import { ShieldAlert } from "lucide-react";

interface SandboxSecurityBannerProps {
  /** Undefined while capabilities haven't loaded yet — renders nothing. */
  sandboxAvailable: boolean | undefined;
}

/**
 * A persistent (non-dismissible, non-auto-expiring) banner shown whenever
 * `capabilities_get` reports the OS sandbox as unavailable — per the IPC
 * spec, a missing sandbox must surface a persistent security banner rather
 * than being silently ignored.
 */
export function SandboxSecurityBanner({ sandboxAvailable }: SandboxSecurityBannerProps) {
  if (sandboxAvailable !== false) return null;

  return (
    <div className="sandbox-security-banner flex items-center gap-2 border-b border-[var(--color-danger)] bg-[var(--color-danger)] px-4 py-2 text-[12px] font-medium text-white">
      <ShieldAlert className="h-4 w-4 shrink-0" />
      <span>
        OS sandbox containment could not be verified on this machine. Write, execute, MCP-local, and
        outside-workspace tools are running without the usual containment guarantees.
      </span>
    </div>
  );
}
