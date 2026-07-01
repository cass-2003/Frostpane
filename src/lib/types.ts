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
}
