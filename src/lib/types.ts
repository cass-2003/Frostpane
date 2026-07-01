export interface DesktopIcon {
  name: string;
  path: string;
  icon_data: string;
  icon_size: number;
  x: number;
  y: number;
  is_shortcut: boolean;
  fence_id?: string | null;
}

export interface FenceStyle {
  bgColor?: string;      // e.g. "rgba(28, 32, 48, 0.42)"
  borderColor?: string;  // e.g. "rgba(255, 255, 255, 0.18)"
  opacity?: number;      // 0-1
  borderRadius?: number; // px
}

export interface FenceTab {
  id: string;
  name: string;
  icon_paths: string[];
}

export interface FenceData {
  id: string;
  title: string;
  emoji: string;
  x: number;
  y: number;
  width: number;
  height: number;
  collapsed: boolean;
  icon_paths: string[];
  viewMode?: "grid" | "list";
  style?: FenceStyle;
  portalPath?: string | null;
  tabs?: FenceTab[];
  activeTab?: string;
}

export interface PortalItem {
  name: string;
  path: string;
  is_dir: boolean;
  extension: string;
  iconData: string;
  size: number;
}

export interface PageData {
  fences: FenceData[];
  iconPositions?: { path: string; x: number; y: number }[];
}

export interface AppSettings {
  iconSize: number; // 32 | 48 | 64 | 96
  autostart: boolean;
  animationLevel: "off" | "basic" | "full";
}
