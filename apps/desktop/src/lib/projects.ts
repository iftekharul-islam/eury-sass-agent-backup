const STORAGE_KEY = 'eury_user_projects_v1';

export interface UserProject {
  path: string;
  name: string;
  openedAt: number;
}

function loadUserProjects(): UserProject[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    return JSON.parse(raw) as UserProject[];
  } catch {
    return [];
  }
}

function saveUserProjects(projects: UserProject[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(projects));
  } catch {
    // ignore quota errors
  }
}

export function listAllProjects(): UserProject[] {
  return loadUserProjects();
}

export function addUserProject(path: string): UserProject {
  const name = path.split('/').filter(Boolean).pop() ?? path;
  const entry: UserProject = { path, name, openedAt: Date.now() };
  const existing = loadUserProjects().filter((p) => p.path !== path);
  saveUserProjects([entry, ...existing]);
  return entry;
}

export function getRecentUserProjects(): UserProject[] {
  return loadUserProjects();
}
