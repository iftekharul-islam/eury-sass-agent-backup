import * as React from "react";
import {
  Home,
  Code2,
  Package,
  Terminal,
  FileText,
  Pencil,
  Plus,
  Globe,
  GitBranch,
  Plug,
  Check,
  X,
  AlertTriangle,
  Search,
  Settings,
  ChevronRight,
  ChevronDown,
  ChevronLeft,
  ChevronUp,
  Square,
  Circle,
  Eye,
  Shield,
  Clock,
  Folder,
  SendHorizontal,
  Paperclip,
  Image,
  AtSign,
  Slash,
  Sparkles,
  Activity,
  ListTodo,
  BookOpen,
  GitCompare,
  RotateCcw,
  Copy,
  Lock,
  Minus,
  Maximize2,
  MoreHorizontal,
  Database,
  User,
  Moon,
  Sun,
  LogOut,
  SlidersHorizontal,
  Play,
  type LucideIcon,
} from "lucide-react";

export const ICON_MAP: Record<string, LucideIcon> = {
  home: Home,
  code: Code2,
  box: Package,
  terminal: Terminal,
  file: FileText,
  pencil: Pencil,
  plus: Plus,
  globe: Globe,
  branch: GitBranch,
  plug: Plug,
  check: Check,
  x: X,
  alert: AlertTriangle,
  search: Search,
  settings: Settings,
  "chev-r": ChevronRight,
  "chev-d": ChevronDown,
  "chev-l": ChevronLeft,
  "chev-u": ChevronUp,
  stop: Square,
  circle: Circle,
  eye: Eye,
  shield: Shield,
  clock: Clock,
  folder: Folder,
  send: SendHorizontal,
  clip: Paperclip,
  image: Image,
  at: AtSign,
  slash: Slash,
  spark: Sparkles,
  activity: Activity,
  list: ListTodo,
  book: BookOpen,
  diff: GitCompare,
  restore: RotateCcw,
  copy: Copy,
  lock: Lock,
  square: Square,
  minus: Minus,
  maximize: Maximize2,
  more: MoreHorizontal,
  database: Database,
  user: User,
  moon: Moon,
  sun: Sun,
  logout: LogOut,
  sliders: SlidersHorizontal,
  play: Play,
};

export interface IconProps extends React.SVGProps<SVGSVGElement> {
  name: string;
  size?: number | 14 | 16 | 18 | 20 | 24 | 28;
  strokeWidth?: number;
}

export function Icon({
  name,
  size,
  className = "",
  style,
  strokeWidth = 1.75,
  ...props
}: IconProps) {
  // Normalize icon key (strip "i-" prefix if present)
  const key = name.startsWith("i-") ? name.slice(2) : name;
  const LucideComp = ICON_MAP[key];

  const sizeClass = size ? `s${size}` : "";
  const finalClass = `i ${sizeClass} ${className}`.trim();

  // Compute pixel size for Lucide component
  const pixelSize = typeof size === "number" ? size : 14;

  if (!LucideComp) {
    // Fallback if an unmapped icon name is passed
    return (
      <Circle
        className={finalClass}
        size={pixelSize}
        strokeWidth={strokeWidth}
        style={style}
        {...props}
      />
    );
  }

  return (
    <LucideComp
      className={finalClass}
      size={pixelSize}
      strokeWidth={strokeWidth}
      style={style}
      {...props}
    />
  );
}

// SvgSprite component kept as a harmless no-op for backward compatibility
export function SvgSprite() {
  return null;
}

// Export Lucide components directly for advanced custom UI composition
export {
  Home,
  Code2,
  Package,
  Terminal,
  FileText,
  Pencil,
  Plus,
  Globe,
  GitBranch,
  Plug,
  Check,
  X,
  AlertTriangle,
  Search,
  Settings,
  ChevronRight,
  ChevronDown,
  ChevronLeft,
  ChevronUp,
  Square,
  Circle,
  Eye,
  Shield,
  Clock,
  Folder,
  SendHorizontal,
  Paperclip,
  Image,
  AtSign,
  Slash,
  Sparkles,
  Activity,
  ListTodo,
  BookOpen,
  GitCompare,
  RotateCcw,
  Copy,
  Lock,
  Minus,
  Maximize2,
  MoreHorizontal,
  Database,
  User,
};
