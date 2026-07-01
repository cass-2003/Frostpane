import { settings, setLocaleInitial } from "./settings.svelte";

type Locale = "en" | "zh";

type Translations = {
  loading: string;
  statusBar: (fences: number, icons: number, page?: number, totalPages?: number) => string;
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
  gridView: string;
  listView: string;
  hideLabels: string;
  autoColor: string;
  archiveToFolder: string;
  archiveSummary: (count: number, dir: string) => string;
  archiveAction: string;
  cancel: string;
  // ContextMenu (icon right-click)
  ctxOpen: string;
  ctxRunAsAdmin: string;
  ctxOpenLocation: string;
  ctxRename: string;
  ctxDelete: string;
  ctxCopyPath: string;
  ctxMoreActions: string;
  // SceneManager
  scenesTitle: string;
  saveScene: string;
  sceneName: string;
  noScenes: string;
  // Onboarding
  welcomeTitle: string;
  welcomeDesc: string;
  getStarted: string;
  chooseLayout: string;
  templateWork: string;
  templatePlay: string;
  templateAllInOne: string;
  skipSetup: string;
  tipRightClick: string;
  tipDragIcons: string;
  tipDoubleClick: string;
  done: string;
  // FenceStyleEditor
  fenceStyle: string;
  resetStyle: string;
  transparency: string;
  borderRadius: string;
  // Rules engine
  rules: string;
  addRule: string;
  ruleName: string;
  ruleCondition: string;
  ruleExtension: string;
  ruleNameContains: string;
  ruleNamePrefix: string;
  ruleTarget: string;
  deleteRule: string;
  noRules: string;
  newPortal: string;
  portalPath: string;
  enterFolderPath: string;
  refreshPortal: string;
  openInExplorer: string;
  // Tabs
  addTab: string;
  renameTab: string;
  deleteTab: string;
  newTab: string;
  // Pages
  nextPage: string;
  prevPage: string;
  addPage: string;
  deletePage: string;
  pages: string;
};

const en: Translations = {
  loading: "Frostpane loading...",
  statusBar: (fences, icons, page, totalPages) =>
    `❄ Frostpane${totalPages && totalPages > 1 ? ` · Page ${page}/${totalPages}` : ""} · ${fences} fences · ${icons} icons`,
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
  gridView: "Grid View",
  listView: "List View",
  hideLabels: "Hide icon labels",
  autoColor: "Auto Color",
  archiveToFolder: "Archive to Folder",
  archiveSummary: (count, dir) => `Move ${count} file${count === 1 ? "" : "s"} to ${dir}`,
  archiveAction: "Archive",
  cancel: "Cancel",
  ctxOpen: "Open",
  ctxRunAsAdmin: "Run as administrator",
  ctxOpenLocation: "Open file location",
  ctxRename: "Rename",
  ctxDelete: "Delete",
  ctxCopyPath: "Copy path",
  ctxMoreActions: "More actions...",
  scenesTitle: "Scenes",
  saveScene: "Save Current",
  sceneName: "Scene name...",
  noScenes: "No saved scenes",
  welcomeTitle: "Welcome to Frostpane",
  welcomeDesc: "Organize your desktop with beautiful glass fences",
  getStarted: "Get Started",
  chooseLayout: "Choose a layout",
  templateWork: "Work",
  templatePlay: "Play",
  templateAllInOne: "All-in-one",
  skipSetup: "Skip — I'll set up manually",
  tipRightClick: "Right-click desktop → New Fence",
  tipDragIcons: "Drag icons into fences",
  tipDoubleClick: "Double-click desktop to show/hide",
  done: "Done!",
  fenceStyle: "Appearance",
  resetStyle: "Reset",
  transparency: "Transparency",
  borderRadius: "Border Radius",
  rules: "Rules...",
  addRule: "Add Rule",
  ruleName: "Rule name",
  ruleCondition: "Condition",
  ruleExtension: "File extension",
  ruleNameContains: "Name contains",
  ruleNamePrefix: "Name prefix",
  ruleTarget: "Target fence",
  deleteRule: "Delete",
  noRules: "No rules defined",
  newPortal: "New Portal Fence",
  portalPath: "Folder path",
  enterFolderPath: "Enter folder path...",
  refreshPortal: "Refresh",
  openInExplorer: "Open in Explorer",
  addTab: "Add Tab",
  renameTab: "Rename Tab",
  deleteTab: "Delete Tab",
  newTab: "New Tab",
  nextPage: "Next Page",
  prevPage: "Previous Page",
  addPage: "Add Page",
  deletePage: "Delete Page",
  pages: "Pages",
};

const zh: Translations = {
  loading: "Frostpane 加载中...",
  statusBar: (fences, icons, page, totalPages) =>
    `❄ 方寸${totalPages && totalPages > 1 ? ` · 第 ${page}/${totalPages} 页` : ""} · ${fences} 个分区 · ${icons} 个图标`,
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
  gridView: "网格视图",
  listView: "列表视图",
  hideLabels: "隐藏图标名称",
  autoColor: "自动取色",
  archiveToFolder: "归档到文件夹",
  archiveSummary: (count, dir) => `将 ${count} 个文件移动到 ${dir}`,
  archiveAction: "归档",
  cancel: "取消",
  ctxOpen: "打开",
  ctxRunAsAdmin: "以管理员身份运行",
  ctxOpenLocation: "打开文件位置",
  ctxRename: "重命名",
  ctxDelete: "删除",
  ctxCopyPath: "复制路径",
  ctxMoreActions: "更多操作...",
  scenesTitle: "场景",
  saveScene: "保存当前",
  sceneName: "场景名称...",
  noScenes: "暂无已保存的场景",
  welcomeTitle: "欢迎使用方寸",
  welcomeDesc: "用毛玻璃分区整理你的桌面",
  getStarted: "开始使用",
  chooseLayout: "选择布局模板",
  templateWork: "工作",
  templatePlay: "娱乐",
  templateAllInOne: "全能",
  skipSetup: "跳过 — 稍后手动设置",
  tipRightClick: "右键桌面 → 新建分区",
  tipDragIcons: "拖拽图标进入分区",
  tipDoubleClick: "双击桌面显示/隐藏分区",
  done: "完成！",
  fenceStyle: "外观",
  resetStyle: "重置",
  transparency: "透明度",
  borderRadius: "圆角",
  rules: "规则...",
  addRule: "添加规则",
  ruleName: "规则名称",
  ruleCondition: "条件",
  ruleExtension: "文件扩展名",
  ruleNameContains: "名称包含",
  ruleNamePrefix: "名称前缀",
  ruleTarget: "目标分区",
  deleteRule: "删除",
  noRules: "暂无规则",
  newPortal: "新建映射分区",
  portalPath: "文件夹路径",
  enterFolderPath: "输入文件夹路径...",
  refreshPortal: "刷新",
  openInExplorer: "在资源管理器中打开",
  addTab: "添加标签页",
  renameTab: "重命名标签页",
  deleteTab: "删除标签页",
  newTab: "新标签页",
  nextPage: "下一页",
  prevPage: "上一页",
  addPage: "添加页面",
  deletePage: "删除页面",
  pages: "页面",
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
