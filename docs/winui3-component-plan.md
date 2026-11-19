# WinUI 3 组件复刻完成计划

复刻范围：**WinUI 3 本体（Microsoft.UI.Xaml）自带控件**。不含 CommunityToolkit 扩展（`DataGrid`、`SettingsCard`、`SettingsExpander` 等属 Toolkit，非本体）。

分类以 **WinUI Gallery 官方分组**为准（`WinUI-Gallery/WinUIGallery/SampleSupport/Data/ControlInfoData.json` 的 `Groups`）。

## 完成条件

- 按 [WinUI Gallery 控件目录](https://github.com/microsoft/WinUI-Gallery/blob/6de91feccadfbb0a796b71caedead5a2284f730e/WinUIGallery/SampleSupport/Data/ControlInfoData.json) 逐项核对范围。补记本体控件缺项，并记录示例页、宿主 API 和外部库控件的归属。
- 对可直接复用的 Slint 控件也执行验收。每项记录对应的实现、原生对照、运行验证和已知差异。现有表格中的「可直接用」仍需核对 WinUI 3 一致性。
- 核对浅色、深色和适用的 Windows 高对比度，检查鼠标、键盘、焦点与可访问状态。实测可运行示例后再标记完成，所有范围内条目通过后结束复刻。

## 实施顺序

1. 补齐已开始的 InfoBar、Expander 和 ContentDialog，先验收内容、浅色与深色视觉和交互。
2. 实现 Flyout 和 MenuFlyout，验证定位、键盘操作、焦点恢复和关闭行为，再复用到 CommandBar、分裂按钮等控件。
3. 实现 NumberBox、AutoSuggestBox、导航、分页、集合与其他待做控件。按下方优先级推进独立组件，并先满足各自的依赖。
4. 补齐现有清单与 Gallery 目录的缺项，覆盖日期、媒体、滚动与布局等类别。
5. 最后统一制作和验收 Windows 高对比度主题，再逐项复验原生控件与 Slint 实现。

## 约定

- 组件放在 `apps/lumatune/ui/components/<分类>/`，按官方分组分子目录（见下）。
- 风格与现有组件一致：`export component`、属性/回调注入、Slint `Palette` 语义色。
- **长度一律偶数**，计算结果取最近偶数（奇点为界取较大偶数，如 1→2、3→4、2.5→2）。
- 图标用 SVG `@image-url(...)` + `colorize` 染色（`apps/lumatune/ui/icons/`）。
- **容器组件避免多个重叠子元素**：`inherits Rectangle` 的自定义组件若有多个绝对重叠的子元素（如全高色条 + 内容布局），其 `preferred-height` 会算不出，放进 `VerticalLayout` 会被压扁。改为**单一布局子元素**（色条作为布局内的窄元素）可让高度由内容撑开。
- 每个组件：编译无警告 + 截图目检 + 至少一处可运行示例验证后再提交。
- 可以省略会触发 Slint 布局重算的过渡动画。起止状态的布局和交互仍需对齐 WinUI 3。
- 状态：⬜ 待做 / 🚧 进行中 / ✅ 完成 / ⏭️ 跳过。

## 目录结构

```
components/
  basic-input/      RadioButtons, RatingControl, ToggleButton, DropDownButton, SplitButton ...
  collections/      TreeView, GridView 增强 ...
  date-time/        CalendarDatePicker, DatePicker, TimePicker ...
  dialogs-flyouts/  ContentDialog(🚧), Flyout, TeachingTip ...
  layout/           Expander(🚧), SplitView ...
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

图例：✅ 已有可直接用 · 🚧 已开始复刻 · 🟡 部分可用需增强 · 🔨 需复刻 · ⚠️ 不建议/难等价

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
| ContentDialog | 🚧 | `components/dialogs-flyouts/content_dialog.slint`，已验收模态焦点、默认键、长正文滚动，以及可取消和延迟的关闭事件 |
| Flyout | 🚧 | `components/dialogs-flyouts/flyout.slint`，支持对齐、贴边翻转、受限尺寸、双轴滚动、可取消关闭和焦点恢复。验收记录见下。 |
| MenuFlyout | 🚧 | `components/menus-toolbars/menu_flyout.slint`，已实现普通与分裂菜单项、选择状态、多级子菜单、上下文入口及可限定作用域的快捷键。 |
| TeachingTip | 🔨 | 引导气泡 |
| Popup | ✅ | Slint `PopupWindow`/基础可用 |

Flyout 对照微软的 [定位源码](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/FlyoutBase_partial.cpp) 和 [呈现模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommonStyles/FlyoutPresenter_themeresources.xaml)。运行 `slint-viewer --style fluent-light examples/winui3/flyout.slint` 可试用布局、外部关闭和溢出滚动。已用 Fluent 浅色软件渲染与深色 FemtoVG 验证定位、首次焦点、Escape、取消外部关闭和双轴滚动。剩余项包括 Acrylic 背景、`FlyoutShowMode`、关闭时向指定元素传递点击、与窗口边界交互的一致性及高对比度。当前用 Acrylic 的纯色回退，指针滚动后焦点移到弹层容器。

MenuFlyout 对照微软的 [菜单规范](https://learn.microsoft.com/windows/apps/develop/ui/controls/menus)、[呈现模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommonStyles/MenuFlyout_themeresources.xaml) 和 [级联定位源码](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/CascadingMenuHelper.cpp)。运行 `cargo run -p lumatune --example menu_flyout` 可试用菜单。示例支持 `--dark`、`--compact`、`--edge` 和 `--scoped-shortcuts`，通过 Cargo 的 `--` 传入。

- 浅色软件渲染和深色 FemtoVG 已验证方向键循环、跳过禁用项和分隔线、Enter 与 Space 激活、勾选和单选互斥、菜单内快捷键、鼠标、Escape 与焦点恢复。小窗口已验证 Home 和 End 滚到当前项。
- 三级菜单已验证向右展开、窗口右缘向左翻转、悬停延迟、Esc 与左方向键逐级收起，以及选中命令或点外部时关闭整串弹层。子菜单重叠 4 px，悬停延迟暂用 400 ms。
- 分裂菜单项的主命令和子菜单入口独立命中。已验证方向键在两部分间切换、Enter 与 Space 松开时激活、收起子菜单后焦点回到入口、没有子项时入口禁用。Windows UI Automation 已核对两个按钮名称、禁用状态和焦点切换。键盘行为同时对照了 [WinUI 分裂菜单项源码](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/SplitMenuFlyoutItem_Partial.cpp)。
- `ContextFlyoutArea` 接收鼠标右键、菜单键与 Shift+F10。右键从指针坐标打开，键盘操作从控件边缘打开。已验证贴边翻转和关闭后恢复焦点。自行处理右键的子控件可用 `request-at` 转发。[Windows 上下文命令规范](https://learn.microsoft.com/windows/apps/develop/ui/controls/collection-commanding)
- `MenuFlyoutAccelerators` 可挂在窗口内容或仅挂在焦点范围内。已验证菜单关闭时执行命令、菜单打开时调用未展开的子项、禁用项不响应，以及限定范围内外的差异。[Windows 键盘快捷键规范](https://learn.microsoft.com/windows/apps/develop/input/keyboard-accelerators)
- 剩余项包括读屏播报、触屏长按与笔输入、从右到左的布局、跟随系统悬停延迟、多窗口边界、Acrylic 和高对比度。

### Layout
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| Grid / StackPanel / RelativePanel / Canvas / Viewbox / Border | ✅ | `GridBox/HorizontalBox/VerticalBox` + 矩形 |
| Expander | 🚧 | `components/layout/expander.slint`，支持文本标题和可组合内容、向上/向下展开 |
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
| InfoBar（可关提示条） | 🚧 | `components/status-info/info_bar.slint`，支持宽窄布局、自定义内容和可区分来源的可取消关闭 |
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

1. 🚧 ContentDialog（`dialogs-flyouts/content_dialog.slint`，模态、鼠标键盘、关闭生命周期已验证，待完成原生视觉和读屏对照）
2. 🚧 InfoBar（`status-info/info_bar.slint`，已验证常用交互及关闭事件，待补自定义操作按钮）
3. 🚧 Expander（`layout/expander.slint`，已验证常用交互，待补可组合标题、内容禁用传递）
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
