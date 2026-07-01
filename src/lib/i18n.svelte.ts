import { settings, setLocaleInitial } from "./settings.svelte";

type Locale = "en" | "zh";

type Translations = {
  loading: string;
  statusBar: (fences: number, icons: number) => string;
  newFence: string;
  showAllFences: string;
  hideAllFences: string;
  sortByName: string;
  settings: string;
  rename: string;
  sortName: string;
  sortType: string;
  appearance: string;
  deleteFence: string;
  settingsTitle: string;
  general: string;
  launchAtStartup: string;
  appearanceSection: string;
  iconSize: string;
  animations: string;
  animOff: string;
  animBasic: string;
  animFull: string;
  language: string;
  about: string;
  aboutDesc: string;
  newFenceTitle: string;
  expand: string;
  collapse: string;
  changeIcon: string;
};

const en: Translations = {
  loading: "Frostpane loading...",
  statusBar: (fences, icons) => `❄ Frostpane · ${fences} fences · ${icons} icons`,
  newFence: "New Fence",
  showAllFences: "Show All Fences",
  hideAllFences: "Hide All Fences",
  sortByName: "Sort Icons by Name",
  settings: "Settings...",
  rename: "Rename",
  sortName: "Sort by Name",
  sortType: "Sort by Type",
  appearance: "Appearance...",
  deleteFence: "Delete Fence",
  settingsTitle: "Settings",
  general: "General",
  launchAtStartup: "Launch at startup",
  appearanceSection: "Appearance",
  iconSize: "Icon size",
  animations: "Animations",
  animOff: "Off",
  animBasic: "Basic",
  animFull: "Full",
  language: "Language",
  about: "About",
  aboutDesc: "A smarter Windows desktop organizer",
  newFenceTitle: "New Fence",
  expand: "Expand",
  collapse: "Collapse",
  changeIcon: "Change icon",
};

const zh: Translations = {
  loading: "Frostpane 加载中...",
  statusBar: (fences, icons) => `❄ 方寸 · ${fences} 个分区 · ${icons} 个图标`,
  newFence: "新建分区",
  showAllFences: "显示所有分区",
  hideAllFences: "隐藏所有分区",
  sortByName: "按名称排序",
  settings: "设置...",
  rename: "重命名",
  sortName: "按名称排序",
  sortType: "按类型排序",
  appearance: "外观...",
  deleteFence: "删除分区",
  settingsTitle: "设置",
  general: "通用",
  launchAtStartup: "开机启动",
  appearanceSection: "外观",
  iconSize: "图标大小",
  animations: "动画效果",
  animOff: "关闭",
  animBasic: "基础",
  animFull: "完整",
  language: "语言",
  about: "关于",
  aboutDesc: "更智能的 Windows 桌面整理工具",
  newFenceTitle: "新分区",
  expand: "展开",
  collapse: "折叠",
  changeIcon: "更换图标",
};

const LOCALES: Record<Locale, Translations> = { en, zh };

function currentLocale(): Locale {
  const l = settings.locale;
  return (l as Locale) in LOCALES ? (l as Locale) : "en";
}

export const t = new Proxy({} as Translations, {
  get(_target, prop: string | symbol) {
    return LOCALES[currentLocale()][prop as keyof Translations];
  },
});

export function setLocale(l: Locale) {
  settings.locale = l;
}

export function initLocale() {
  const detected: Locale = navigator.language.startsWith("zh") ? "zh" : "en";
  setLocaleInitial(detected);
}
