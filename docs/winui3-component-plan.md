# WinUI 3 组件复刻目标

复刻范围：**WinUI 3 本体（Microsoft.UI.Xaml）自带控件**。不含 CommunityToolkit 扩展（`DataGrid`、`SettingsCard`、`SettingsExpander` 等属 Toolkit，非本体）。

分类以 **WinUI Gallery 官方分组**为准（`WinUI-Gallery/WinUIGallery/SampleSupport/Data/ControlInfoData.json` 的 `Groups`）。

## 约定

- 组件放在 `apps/lumatune/ui/components/<分类>/`，按官方分组分子目录（见下）。
- 风格与现有组件一致：`export component`、属性/回调注入、Slint `Palette` 语义色。
- **长度一律偶数**，计算结果取最近偶数（奇点为界取较大偶数，如 1→2、3→4、2.5→2）。
- 图标用 SVG `@image-url(...)` + `colorize` 染色（`apps/lumatune/ui/icons/`）。
- 每个组件：编译无警告 + 截图目检 + 至少一处可运行示例验证后再提交。
- 状态：⬜ 待做 / 🚧 进行中 / ✅ 完成 / ⏭️ 跳过。

## 目录结构

```
components/
  basic-input/      RadioButtons, RatingControl, ToggleButton, DropDownButton, SplitButton ...
  collections/      TreeView, GridView 增强 ...
  date-time/        CalendarDatePicker, DatePicker, TimePicker ...
  dialogs-flyouts/  ContentDialog(✅), Flyout, TeachingTip ...
  layout/           Expander, SplitView ...
  media/            PersonPicture, AnimatedVisualPlayer ...
  menus-toolbars/   CommandBar, MenuFlyout, AppBarButton ...
  motion/           Connected Animation 等（按需）
  navigation/       NavRail(自研,✅), BreadcrumbBar(✅), NavigationView, TabView, Pivot
  scrolling/        PipsPager, AnnotatedScrollBar ...
  shell/            页面壳 / Frame ...
  status-info/      InfoBar, InfoBadge, ProgressRing ...
  text/             NumberBox, AutoSuggestBox, PasswordBox ...
```

> 现有组件已归位：`navigation/nav_rail.slint`、`navigation/breadcrumb_bar.slint`、`dialogs-flyouts/content_dialog.slint`。

## Slint 1.18.1 已有（可直接用）

`Button, CheckBox, ComboBox, GroupBox, LineEdit, RadioGroup, ListView, StandardListView, ProgressIndicator, Slider, SpinBox, Spinner, TabWidget, Switch, TextEdit, TimePickerPopup, DatePickerPopup, ScrollView, VerticalBox/HorizontalBox/GridBox, Palette`

## 官方分组 × 可行性

图例：✅ 已有可直接用 · 🟡 部分可用需增强 · 🔨 需复刻 · ⚠️ 不建议/难等价

### Basic input
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| Button / RepeatButton | ✅ | |
| CheckBox | ✅ | |
| ComboBox | ✅ | |
| Slider | ✅ | |
| RadioButton | 🟡 | `RadioGroup` 有；便捷分组 `RadioButtons` 待封装 |
| ToggleSwitch | 🟡 | `Switch` 有，样式不完全同 |
| ToggleButton / DropDownButton / SplitButton / ToggleSplitButton | 🔨 | 皆无 |
| HyperlinkButton | 🔨 | 无 |
| RatingControl | 🔨 | |
| ColorPicker | ⚠️ | 复杂，暂不做 |

### Collections
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| ListView | ✅ | |
| GridView | 🟡 | `ListView`/`for`+wrap 可拼 |
| ItemsRepeater | 🟡 | `for`/`ListView` 覆盖多数 |
| TreeView | 🔨 | 无 |
| FlipView | 🔨 | 轮播 |

### Date & time
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| CalendarDatePicker / DatePicker / TimePicker | 🟡 | 底层 `DatePickerPopup/TimePickerPopup` 有弹层，缺完整 picker |
| CalendarView | 🔨 | |

### Dialogs & flyouts
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| ContentDialog | ✅ | `components/dialogs-flyouts/content_dialog.slint` |
| Flyout / MenuFlyout | 🔨 | 任意锚点浮层 |
| TeachingTip | 🔨 | 引导气泡 |
| Popup | ✅ | Slint `PopupWindow`/基础可用 |

### Layout
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| Grid / StackPanel / RelativePanel / Canvas / Viewbox / Border | ✅ | `GridBox/HorizontalBox/VerticalBox` + 矩形 |
| Expander | 🔨 | 折叠区块，高价值 |
| SplitView | 🔨 | 抽屉 + 遮罩 |
| VariableSizedWrapGrid / WrapPanel | 🟡 | 可用布局近似 |

### Navigation
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| BreadcrumbBar | ✅ | `components/navigation/breadcrumb_bar.slint` |
| NavigationView | 🔨 | 项目自研 `NavRail` 覆盖部分 |
| TabView（可增删/关闭） | 🟡 | `TabWidget` 静态标签，无关闭/新建 |
| Pivot | 🔨 | 可用 `TabWidget` 近似 |
| SelectorBar | 🔨 | |

### Status & info
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| InfoBar（可关提示条） | 🔨 | 高价值 |
| InfoBadge（角标） | 🔨 | |
| ProgressBar | 🟡 | `ProgressIndicator` |
| ProgressRing（环形） | 🔨 | `Spinner` 非环形进度 |
| ToolTip | 🟡 | 基础可用，样式弱 |

### Scrolling
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| ScrollView / ScrollViewer | ✅ | `ScrollView` |
| PipsPager | 🔨 | 分页点 |
| AnnotatedScrollBar | 🔨 | |
| PagerControl | 🔨 | |
| SemanticZoom | 🔨 | |

### Menus & toolbars
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| MenuBar / MenuBarItem | 🟡 | 公开有 `MenuBar` |
| CommandBar / AppBarButton / AppBarToggleButton / AppBarSeparator | 🔨 | 高价值（含 overflow …） |
| CommandBarFlyout | 🔨 | |
| SwipeControl | 🔨 | |

### Text
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| TextBlock | ✅ | `Text` |
| TextBox / LineEdit | 🟡 | `LineEdit`；password 需掩码封装 |
| PasswordBox | 🔨 | 掩码封装 |
| NumberBox（± 步进） | 🔨 | 高价值；`SpinBox` 可包装 |
| AutoSuggestBox（建议下拉） | 🔨 | 高价值 |
| RichTextBlock / RichEditBox | ⚠️ | 富文本支持弱 |

### Media
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| Image | ✅ | |
| PersonPicture（头像） | 🔨 | 低难度 |
| AnimatedVisualPlayer（Lottie） | ⚠️ | 依赖 Lottie 运行时 |
| WebView2 / MediaPlayerElement | ⚠️ | 宿主能力，难等价 |

### Motion / Windowing / System / Shell / Styles
- Motion：`Connected Animation`、`Page/Theme Transitions` → 🔨/⚠️（Slint 动画能力有限，按需）。
- Windowing：`Window`/`TitleBar` → 🟡（Slint 窗口能力为主）。
- System：`Clipboard`/`Storage pickers` → 需宿主（Rust 侧）能力，非纯组件。
- Shell：通知/Badge/JumpList → 宿主能力，非纯组件。
- Styles：`AcrylicBrush`/`Mica`/`ThemeShadow`/`AnimatedIcon` → 🟡/🔨（背景与阴影可按需封装）。

## 复刻优先级（实用 + Slint 缺）

1. ✅ ContentDialog（`dialogs-flyouts/content_dialog.slint`）
2. ⬜ InfoBar（status-info）
3. ⬜ Expander（layout）
4. ⬜ CommandBar（menus-toolbars）
5. ⬜ NumberBox / AutoSuggestBox（text）
6. ⬜ NavigationView（navigation，或在现有 NavRail 上补齐）
7. ⬜ TabView（navigation，在 TabWidget 上补关闭/新建）
8. ⬜ InfoBadge / PipsPager / PersonPicture（status-info / scrolling / media）
9. ⬜ Flyout / MenuFlyout / TeachingTip（dialogs-flyouts）
10. ⬜ TreeView / RatingControl / ProgressRing / HyperlinkButton

## 每个组件的验收标准

- 编译通过且无警告；`cargo test` 通过。
- 软件渲染截图目检：布局、配色、交互态（hover/press/disabled/暗色）符合 WinUI 3 视觉。
- 至少一处可运行示例（临时插入预览、验证后移除）证明实际渲染正确。
- 遵循本文件顶部约定（分类目录、偶数长度、Palette、SVG 图标）。
