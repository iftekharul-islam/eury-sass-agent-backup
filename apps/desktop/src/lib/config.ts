/** NestJS backend — serves /agent/v1/* */
export function getAgentApiUrl(): string {
  const raw =
    import.meta.env.VITE_EURY_AGENT_API_URL ??
    import.meta.env.VITE_AGENT_API_URL ??
    "http://localhost:3001";
  const trimmed = String(raw).replace(/\/$/, "");
  return trimmed.replace(/\/agent\/v1$/, "");
}

/** Next.js web app — serves /agent/authorize */
export function getAgentWebUrl(): string {
  return (import.meta.env.VITE_AGENT_WEB_URL ?? "http://localhost:3000").replace(
    /\/$/,
    "",
  );
}

export function getAgentAuthorizeUrl(): string {
  const locale = import.meta.env.VITE_AGENT_WEB_LOCALE ?? "en";
  return `${getAgentWebUrl()}/${locale}/agent/authorize`;
}
