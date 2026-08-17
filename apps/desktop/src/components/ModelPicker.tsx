import * as React from 'react';
import { createPortal } from 'react-dom';
import { Icon } from './Icons';
import { getEuryModels, type EuryModelCatalogItem } from '../lib/chat';
import {
  EFFORT_OPTIONS,
  effortLabelFor,
  type EffortLevel,
} from '../lib/effort';

const FEATURED_MODEL_IDS = [
  'claude-fable-5',
  'claude-opus-5',
  'gpt-5.5',
  'gemini-3.6',
] as const;

export interface ModelPickerProps {
  value: { provider: string; modelId: string; label: string };
  onChange: (model: { provider: string; modelId: string; label: string }) => void;
  disabled?: boolean;
  effort?: EffortLevel;
  onEffortChange?: (effort: EffortLevel) => void;
}

function modelDescription(model: EuryModelCatalogItem): string {
  if (model.mock) return 'Preview routing';
  return model.tier === 'fast' ? 'Fast responses' : 'Reasoning';
}

export function ModelPicker({
  value,
  onChange,
  disabled,
  effort,
  onEffortChange,
}: ModelPickerProps) {
  const [open, setOpen] = React.useState(false);
  const [models, setModels] = React.useState<EuryModelCatalogItem[]>([]);
  const [loading, setLoading] = React.useState(false);
  const [error, setError] = React.useState<string | null>(null);
  const [sidePanel, setSidePanel] = React.useState<'more' | 'effort' | null>(null);
  const anchorRef = React.useRef<HTMLSpanElement>(null);
  const menuRef = React.useRef<HTMLDivElement>(null);
  const [layout, setLayout] = React.useState<{ top: number; left: number } | null>(null);

  const showEffort = effort !== undefined && Boolean(onEffortChange);

  React.useEffect(() => {
    let cancelled = false;
    setLoading(true);
    getEuryModels()
      .then((res) => {
        if (!cancelled) {
          setModels(res.models);
          setError(null);
        }
      })
      .catch((e: Error) => {
        if (!cancelled) setError(e.message);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const featuredModels = React.useMemo(() => {
    const featured = FEATURED_MODEL_IDS.map((id) => models.find((m) => m.id === id)).filter(
      (m): m is EuryModelCatalogItem => Boolean(m),
    );
    return featured.length > 0 ? featured : models.slice(0, 4);
  }, [models]);

  const moreModels = React.useMemo(() => {
    const featuredIds = new Set(featuredModels.map((m) => m.id));
    return models.filter((m) => !featuredIds.has(m.id));
  }, [featuredModels, models]);

  const updateLayout = React.useCallback(() => {
    const anchor = anchorRef.current?.getBoundingClientRect();
    const menuHeight = menuRef.current?.offsetHeight ?? 360;
    if (!anchor) return;
    const panelWidth = 296;
    const gap = 8;
    const margin = 12;
    let left = anchor.right - panelWidth;
    left = Math.max(margin, Math.min(left, window.innerWidth - panelWidth - margin));
    let top = anchor.top - menuHeight - gap;
    if (top < margin) {
      top = Math.min(anchor.bottom + gap, window.innerHeight - menuHeight - margin);
    }
    setLayout({ top, left });
  }, []);

  React.useLayoutEffect(() => {
    if (!open) {
      setLayout(null);
      setSidePanel(null);
      return;
    }
    updateLayout();
    const menuEl = menuRef.current;
    if (!menuEl) return;
    const observer = new ResizeObserver(() => updateLayout());
    observer.observe(menuEl);
    return () => observer.disconnect();
  }, [open, updateLayout, featuredModels.length, moreModels.length, showEffort]);

  React.useEffect(() => {
    if (!open) return;
    const handlePointerDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (anchorRef.current?.contains(target) || menuRef.current?.contains(target)) return;
      setOpen(false);
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setOpen(false);
    };
    document.addEventListener('mousedown', handlePointerDown);
    document.addEventListener('keydown', handleKeyDown);
    window.addEventListener('resize', updateLayout);
    window.addEventListener('scroll', updateLayout, true);
    return () => {
      document.removeEventListener('mousedown', handlePointerDown);
      document.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('resize', updateLayout);
      window.removeEventListener('scroll', updateLayout, true);
    };
  }, [open, updateLayout]);

  const handleSelect = (model: EuryModelCatalogItem) => {
    onChange({
      provider: model.apiProvider,
      modelId: model.id,
      label: model.version,
    });
    setOpen(false);
    setSidePanel(null);
  };

  const handleEffortSelect = (level: EffortLevel) => {
    onEffortChange?.(level);
  };

  const triggerLabel = showEffort && effort
    ? `${value.label} ${effortLabelFor(effort)}`
    : value.label;

  const menu =
    open && typeof document !== 'undefined' ? (
      <div
        ref={menuRef}
        className="model-picker-menu"
        style={{
          position: 'fixed',
          top: layout?.top ?? -9999,
          left: layout?.left ?? -9999,
          zIndex: 300,
          visibility: layout ? 'visible' : 'hidden',
        }}
        onMouseLeave={() => setSidePanel(null)}
      >
        <div className="model-picker-panel">
          {loading && <div className="model-picker-status">Loading models…</div>}
          {error && <div className="model-picker-status error">{error}</div>}

          {!loading &&
            featuredModels.map((model) => {
              const selected = model.id === value.modelId;
              return (
                <button
                  key={model.id}
                  type="button"
                  className={`model-picker-row ${selected ? 'selected' : ''}`}
                  onClick={() => handleSelect(model)}
                >
                  <div className="model-picker-row-main">
                    <span className="model-picker-name">{model.version}</span>
                    <span className="model-picker-desc">{modelDescription(model)}</span>
                  </div>
                  {selected ? <Icon name="check" size={14} /> : null}
                </button>
              );
            })}

          {(showEffort || moreModels.length > 0) ? (
            <div className="model-picker-footer">
              {showEffort && effort ? (
                <button
                  type="button"
                  className={`model-picker-row compact ${sidePanel === 'effort' ? 'selected' : ''}`}
                  onMouseEnter={() => setSidePanel('effort')}
                  onFocus={() => setSidePanel('effort')}
                >
                  <span className="model-picker-name">Effort</span>
                  <span className="model-picker-row-trailing">
                    {effortLabelFor(effort)}
                    <Icon name="chev-r" size={14} />
                  </span>
                </button>
              ) : null}

              {moreModels.length > 0 ? (
                <button
                  type="button"
                  className={`model-picker-row compact ${sidePanel === 'more' ? 'selected' : ''}`}
                  onMouseEnter={() => setSidePanel('more')}
                  onFocus={() => setSidePanel('more')}
                >
                  <span className="model-picker-name">More models</span>
                  <Icon name="chev-r" size={14} />
                </button>
              ) : null}
            </div>
          ) : null}

          {sidePanel === 'effort' && showEffort && effort ? (
            <div
              className="model-picker-side"
              onMouseEnter={() => setSidePanel('effort')}
            >
              <div className="model-picker-side-scroll model-picker-effort-panel">
                <p className="model-picker-effort-hint">
                  Higher effort means more thorough responses, but takes longer and uses your limits faster.
                </p>
                {(Object.keys(EFFORT_OPTIONS) as EffortLevel[]).map((level) => {
                  const selected = level === effort;
                  return (
                    <button
                      key={level}
                      type="button"
                      className={`model-picker-row compact ${selected ? 'selected' : ''}`}
                      onClick={() => handleEffortSelect(level)}
                    >
                      <span className="model-picker-effort-label">
                        {EFFORT_OPTIONS[level].label}
                        {level === 'medium' ? (
                          <span className="model-picker-effort-badge">Default</span>
                        ) : null}
                      </span>
                      {selected ? <Icon name="check" size={14} /> : null}
                    </button>
                  );
                })}
              </div>
            </div>
          ) : null}

          {sidePanel === 'more' ? (
            <div
              className="model-picker-side"
              onMouseEnter={() => setSidePanel('more')}
            >
              <div className="model-picker-side-scroll">
                {moreModels.map((model) => {
                  const selected = model.id === value.modelId;
                  return (
                    <button
                      key={model.id}
                      type="button"
                      className={`model-picker-row compact ${selected ? 'selected' : ''}`}
                      onClick={() => handleSelect(model)}
                    >
                      <span className="model-picker-name">{model.version}</span>
                      {selected ? <Icon name="check" size={14} /> : null}
                    </button>
                  );
                })}
              </div>
            </div>
          ) : null}
        </div>
      </div>
    ) : null;

  return (
    <>
      <span
        ref={anchorRef}
        className={`select model-picker-trigger ${disabled ? 'disabled' : ''}`}
        onClick={() => !disabled && setOpen((p) => !p)}
        role="button"
        tabIndex={disabled ? -1 : 0}
        onKeyDown={(e) => {
          if (!disabled && (e.key === 'Enter' || e.key === ' ')) {
            e.preventDefault();
            setOpen((p) => !p);
          }
        }}
      >
        <span className="model-picker-trigger-label">{triggerLabel}</span>
        <Icon name="chev-d" />
      </span>
      {menu ? createPortal(menu, document.body) : null}
    </>
  );
}
