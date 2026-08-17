import { describe, expect, it } from 'vitest';
import { detectTaskIntent, buildTaskEnvelope } from './system-prompt';

describe('detectTaskIntent', () => {
  it('detects scaffold from English', () => {
    expect(detectTaskIntent('create a Next.js app with Tailwind')).toBe('scaffold_project');
  });

  it('detects scaffold from Banglish', () => {
    expect(detectTaskIntent('ekta nextjs app banate')).toBe('scaffold_project');
  });

  it('detects run dev server', () => {
    expect(detectTaskIntent('run the app locally')).toBe('run_dev_server');
  });

  it('detects fix intent', () => {
    expect(detectTaskIntent('fix the typescript error')).toBe('fix_error');
  });

  it('builds task envelope', () => {
    const envelope = buildTaskEnvelope('create a react app');
    expect(envelope).toContain('Intent: scaffold_project');
    expect(envelope).toContain('create a react app');
  });
});
