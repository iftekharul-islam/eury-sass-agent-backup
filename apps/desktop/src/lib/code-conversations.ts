const STORAGE_KEY = 'eury_code_conversations_v1';

export interface CodeConversation {
  id: string;
  title: string;
  workspacePath: string;
  updatedAt: number;
}

function loadAll(): Record<string, CodeConversation[]> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    return JSON.parse(raw) as Record<string, CodeConversation[]>;
  } catch {
    return {};
  }
}

function saveAll(data: Record<string, CodeConversation[]>) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
  } catch {
    // ignore quota errors
  }
}

export function listCodeConversations(workspacePath: string): CodeConversation[] {
  const items = loadAll()[workspacePath] ?? [];
  return [...items].sort((a, b) => b.updatedAt - a.updatedAt);
}

export function createCodeConversation(
  workspacePath: string,
  title = 'New conversation',
): CodeConversation {
  const entry: CodeConversation = {
    id: crypto.randomUUID(),
    title,
    workspacePath,
    updatedAt: Date.now(),
  };
  const all = loadAll();
  const next = [entry, ...(all[workspacePath] ?? [])];
  all[workspacePath] = next;
  saveAll(all);
  return entry;
}

export function touchCodeConversation(
  workspacePath: string,
  conversationId: string,
  title?: string,
): CodeConversation | null {
  const all = loadAll();
  const items = all[workspacePath];
  if (!items?.length) return null;
  const index = items.findIndex((item) => item.id === conversationId);
  if (index < 0) return null;
  const updated: CodeConversation = {
    ...items[index],
    updatedAt: Date.now(),
    ...(title?.trim() ? { title: title.trim() } : {}),
  };
  items[index] = updated;
  all[workspacePath] = items;
  saveAll(all);
  return updated;
}

export function ensureDefaultCodeConversation(workspacePath: string): CodeConversation {
  const existing = listCodeConversations(workspacePath);
  if (existing.length > 0) return existing[0];
  return createCodeConversation(workspacePath);
}

function titleFromMessage(text: string): string {
  const trimmed = text.trim().replace(/\s+/g, ' ');
  if (!trimmed) return 'New conversation';
  return trimmed.length > 72 ? `${trimmed.slice(0, 72)}…` : trimmed;
}

export function registerCodeConversationMessage(
  workspacePath: string,
  conversationId: string,
  message: string,
): CodeConversation {
  const existing = touchCodeConversation(workspacePath, conversationId);
  if (existing) {
    const isDefaultTitle =
      existing.title === 'New conversation' || existing.title.trim().length === 0;
    if (isDefaultTitle) {
      const titled = touchCodeConversation(
        workspacePath,
        conversationId,
        titleFromMessage(message),
      );
      if (titled) return titled;
    }
    return existing;
  }
  return createCodeConversation(workspacePath, titleFromMessage(message));
}
