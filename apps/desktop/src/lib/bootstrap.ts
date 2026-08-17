import { getEuryModels } from './chat';
import { SettingsStore } from './settings';

function isFactoryDefaultModel(): boolean {
  const current = SettingsStore.get().model;
  return (
    current.activeProvider === 'OpenAI' &&
    current.activeModelId === 'gpt-4o-mini' &&
    current.activeModelLabel === 'GPT-4o mini'
  );
}

/** Seed the model picker once — never overwrite a saved or user-selected model. */
export async function bootstrapDefaultModel(): Promise<void> {
  if (!isFactoryDefaultModel()) return;

  try {
    const resp = await getEuryModels();
    const first = resp.models[0];
    if (!first) return;
    SettingsStore.updateModel({
      activeProvider: first.apiProvider,
      activeModelId: first.id,
      activeModelLabel: first.version,
    });
  } catch (err) {
    console.warn('Failed to bootstrap default model from gateway:', err);
  }
}
