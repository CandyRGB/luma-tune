# WinUI 3 组件复刻完成计划

复刻范围：**WinUI 3 本体（Microsoft.UI.Xaml）自带控件**。不含 CommunityToolkit 扩展（`DataGrid`、`SettingsCard`、`SettingsExpander` 等属 Toolkit，非本体）。

分类以 **WinUI Gallery 官方分组**为准（`WinUI-Gallery/WinUIGallery/SampleSupport/Data/ControlInfoData.json` 的 `Groups`）。

## 完成条件

- 按 [WinUI Gallery 控件目录](https://github.com/microsoft/WinUI-Gallery/blob/6de91feccadfbb0a796b71caedead5a2284f730e/WinUIGallery/SampleSupport/Data/ControlInfoData.json) 逐项核对范围。补记本体控件缺项，并记录示例页、宿主 API 和外部库控件的归属。
- 对可直接复用的 Slint 控件也执行验收。每项记录对应的实现、原生对照、运行验证和已知差异。现有表格中的「可直接用」仍需核对 WinUI 3 一致性。
- 在浅色主题下核对视觉，检查鼠标、键盘、焦点与可访问状态。深色和 Windows 高对比度免于专项验证。实测可运行示例后再标记完成，所有范围内条目通过后结束复刻。

## 实施顺序

1. 补齐已开始的 InfoBar、Expander 和 ContentDialog，先验收内容、浅色视觉和交互。
2. 实现 Flyout 和 MenuFlyout，验证定位、键盘操作、焦点恢复和关闭行为，再复用到 CommandBar、分裂按钮等控件。
3. 实现 NumberBox、AutoSuggestBox、导航、分页、集合与其他待做控件。按下方优先级推进独立组件，并先满足各自的依赖。
4. 补齐现有清单与 Gallery 目录的缺项，覆盖日期、媒体、滚动与布局等类别。
5. 最后统一制作 Windows 高对比度主题。

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
| ToggleButton | 🔨 | 待校对 Slint `Button.checkable` 与 WinUI 的视觉和交互态。 |
| DropDownButton / SplitButton / ToggleSplitButton | 🚧 | `components/basic-input/flyout_button.slint`，可接入 Flyout 或 MenuFlyout，支持键鼠、禁用态和选中状态。 |
| HyperlinkButton | 🔨 | 无 |
| RatingControl | 🔨 | |
| ColorPicker | ⚠️ | 复杂，暂不做 |

运行 `cargo run -p lumatune --example flyout_buttons` 可试用三个按钮，附加 `-- --dark` 切换至深色。已验证鼠标点击、Enter 与 Space、分裂按钮的 F4 与 Alt+↓、弹层关闭后的焦点恢复、禁用态，以及 ToggleSplitButton 用主区域切换状态、用箭头开菜单。Windows UI Automation 已实测展开、调用和切换。AccessKit 通过独立子节点提供分裂按钮的主命令。读屏、原生收起动作、图标动效和触屏仍待验收。[DropDownButton 模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/DropDownButton/DropDownButton.xaml) · [SplitButton 模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/SplitButton/SplitButton.xaml) · [SplitButton 键盘实现](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/SplitButton/SplitButton.cpp)

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

运行 `slint-viewer --style fluent-light examples/winui3/flyout.slint` 可试用布局、外部关闭和溢出滚动。已用 Fluent 浅色软件渲染与深色 FemtoVG 验证定位、首次焦点、Escape、取消外部关闭和双轴滚动。剩余项包括 Acrylic 背景、`FlyoutShowMode`、关闭时向指定元素传递点击和窗口边界交互。当前用 Acrylic 的纯色回退，指针滚动后焦点移到弹层容器。[WinUI 定位源码](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/FlyoutBase_partial.cpp) · [WinUI 呈现模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommonStyles/FlyoutPresenter_themeresources.xaml)

MenuFlyout 对照微软的 [菜单规范](https://learn.microsoft.com/windows/apps/develop/ui/controls/menus)、[呈现模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommonStyles/MenuFlyout_themeresources.xaml) 和 [级联定位源码](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/CascadingMenuHelper.cpp)。运行 `cargo run -p lumatune --example menu_flyout` 可试用菜单。示例支持 `--dark`、`--compact`、`--edge` 和 `--scoped-shortcuts`，通过 Cargo 的 `--` 传入。

- 浅色软件渲染和深色 FemtoVG 已验证方向键循环、跳过禁用项和分隔线、Enter 与 Space 激活、勾选和单选互斥、菜单内快捷键、鼠标、Escape 与焦点恢复。小窗口已验证 Home 和 End 滚到当前项。
- 三级菜单已验证向右展开、窗口右缘向左翻转、悬停延迟、Esc 与左方向键逐级收起，以及选中命令或点外部时关闭整串弹层。子菜单重叠 4 px，悬停延迟暂用 400 ms。
- 分裂菜单项的主命令和子菜单入口独立命中。已验证方向键在两部分间切换、Enter 与 Space 松开时激活、收起子菜单后焦点回到入口、没有子项时入口禁用。Windows UI Automation 已核对两个按钮名称、禁用状态和焦点切换。键盘行为同时对照了 [WinUI 分裂菜单项源码](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/SplitMenuFlyoutItem_Partial.cpp)。
- `ContextFlyoutArea` 接收鼠标右键、菜单键与 Shift+F10。右键从指针坐标打开，键盘操作从控件边缘打开。已验证贴边翻转和关闭后恢复焦点。自行处理右键的子控件可用 `request-at` 转发。[Windows 上下文命令规范](https://learn.microsoft.com/windows/apps/develop/ui/controls/collection-commanding)
- `MenuFlyoutAccelerators` 可挂在窗口内容或仅挂在焦点范围内。已验证菜单关闭时执行命令、菜单打开时调用未展开的子项、禁用项不响应，以及限定范围内外的差异。[Windows 键盘快捷键规范](https://learn.microsoft.com/windows/apps/develop/input/keyboard-accelerators)
- 剩余项包括读屏播报、触屏长按与笔输入、从右到左的布局、跟随系统悬停延迟、多窗口边界和 Acrylic。

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
| CommandBar / AppBarButton / AppBarToggleButton / AppBarSeparator | 🚧 | `components/menus-toolbars/command_bar.slint` 与 `app_bar_button.slint`，支持主次命令、动态溢出、自定义内容和附属菜单。 |
| CommandBarFlyout | 🚧 | `components/menus-toolbars/command_bar_flyout.slint`，复用 AppBar 元素并支持标准、瞬态和指针移远关闭模式。 |
| SwipeControl | 🔨 | |

运行 `cargo run -p lumatune --example command_bar` 可试用命令栏。附加 `-- --narrow --bottom` 可查看窄窗口和向上展开，`-- --narrow --ordered` 可查看指定顺序的成组迁移，`-- --app-bar-buttons` 可单独试用三种 AppBar 元素。已验证浅色下的紧凑与最小状态、向上和向下展开、方向键、Esc、外部点击关闭、快捷键、溢出区内滚动和附属菜单。实际调整窗口大小时，切换命令迁入和移出溢出区后保留选中状态。Windows UI Automation 已实测展开、调用和切换。[Microsoft CommandBar 规范](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/command-bar) · [WinUI 溢出与键盘源码](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/CommandBar_Partial.cpp) · [命令栏模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommonStyles/CommandBar_themeresources.xaml) · [按钮模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommonStyles/AppBarButton_themeresources.xaml)

展开状态使用 Acrylic 的纯色回退，溢出区在窗口内绘制。长标签换行、`AppBarElementContainer` 和分裂命令、跨窗口弹层、触屏和手柄、RTL 目检及读屏播报仍待补齐或验收。

运行 `cargo run -p lumatune --example command_bar_flyout` 可试用两种打开方式。附加 `-- --edge` 可试用向上展开，`-- --pointer-dismiss` 可试用指针移远关闭，`-- --always-expanded`、`-- --primary-only` 和 `-- --secondary-only` 可试用其余命令布局。浅色已验证标准模式起始焦点、瞬态保留输入焦点、省略号双向切换、Tab 与方向键、外部点击、指针距弹层超过 80px 后关闭、上下文键和附属菜单。执行次命令可关闭浮层并恢复到调用方指定的焦点。Windows UI Automation 已实测展开、切换和调用。[Microsoft CommandBarFlyout 规范](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/command-bar-flyout) · [WinUI 键盘逻辑](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommandBarFlyout/CommandBarFlyoutCommandBar.cpp) · [WinUI 呈现模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommandBarFlyout/CommandBarFlyout_themeresources.xaml)

当前浮层随宿主窗口绘制，使用 Acrylic 的纯色回退。宿主通过 `restore-focus` 指定恢复元素，用 `CommandBarFlyoutKeyScope` 处理瞬态下的 Esc。原生菜单自动化角色、跨窗口定位、自定义命令容器、触屏和手柄仍待补齐或验收。

### Text
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| TextBlock | ✅ | `Text` |
| TextBox / LineEdit | 🟡 | `LineEdit`；password 需掩码封装 |
| PasswordBox | 🔨 | 掩码封装 |
| TextCommandBarFlyout | 🚧 | `components/text/text_command_bar_flyout.slint`，根据文本控件、选区、剪贴板和编辑历史显示命令。 |
| NumberBox | 🚧 | `components/text/number_box.slint`，支持隐藏、内联与紧凑步进，以及表达式、空值、范围和验证模式。 |
| AutoSuggestBox（建议下拉） | 🚧 | `components/text/auto_suggest_box.slint`，支持异步筛选、键鼠选择、文本预览及查询提交。 |
| RichTextBlock / RichEditBox | ⚠️ | 富文本支持弱 |

运行 `cargo run -p lumatune --example text_command_bar_flyout` 可试用可编辑文本、只读文本和密码输入框。右键、菜单键和 Shift+F10 打开上下文菜单，“Show selection tools” 演示选区工具栏。浅色已验证选区在菜单夺取焦点后仍可见，Esc 后恢复编辑焦点，次命令执行后收起，以及撤销、重做和系统剪贴板上的 Unicode 剪切、复制和粘贴。只读示例仅显示复制和全选，密码示例仅显示可用的粘贴和全选。Windows UI Automation 已验证调用全选和展开次命令。[Microsoft 文档及截图](https://learn.microsoft.com/windows/apps/develop/ui/controls/command-bar-flyout) · [WinUI 命令规则](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommandBarFlyout/TextCommandBarFlyout.cpp)

编辑器负责提供选区、可粘贴状态和撤销重做能力。示例管理逐步编辑历史。实机触控自动打开及选区定位、富文本编辑器接入、拼写检查和读屏播报仍待补齐或验收。

运行 `cargo run -p lumatune --example number_box` 可试用 NumberBox，附加 `-- --dark` 切换至深色。已验证 Enter 与失焦提交、Esc 恢复、上下键和 PageUp/PageDown、焦点内滚轮、内联与紧凑步进、长按连续步进、环绕、两种验证模式、范围限制、长输入光标和程序设值。Windows UI Automation 已实测 Spinner 类型、范围属性和设值。[Microsoft NumberBox 规范](https://learn.microsoft.com/windows/apps/develop/ui/controls/number-box) · [WinUI 控件实现](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/NumberBox/NumberBox.cpp) · [WinUI 呈现模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/NumberBox/NumberBox.xaml)

当前公开数值使用 Slint 的 32 位浮点，空值由 `has-value` 表示。十进制分隔符跟随 Slint 区域设置，支持自定义解析和格式化回调。仍需补齐 WinUI `Double` 的精度和地区格式、编辑器右键菜单、紧凑步进面板跨窗口绘制和读屏播报。

运行 `cargo run -p lumatune --example auto_suggest_box` 可试用 AutoSuggestBox，附加 `-- --above` 可将编辑器放在窗口底部，触发建议列表向上展开。已验证键盘选择、Esc 和 Tab 恢复原始文本、Enter 和鼠标提交、搜索图标提交、禁用建议文本预览、清空、禁用和异步筛选。Windows UI Automation 已实测编辑框类型与设值、展开状态和列表项类型。共享输入框样式后复验了 NumberBox 的键鼠交互和长文本光标滚动。[Microsoft AutoSuggestBox 规范](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/auto-suggest-box) · [WinUI 控件实现](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/AutoSuggestBox_Partial.cpp) · [WinUI 呈现模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/AutoSuggestBox/AutoSuggestBox_themeresources.xaml)

建议弹层当前在父容器内绘制，使用 Acrylic 的纯色回退。跨窗口与父容器裁剪、建议项模板、IME 候选窗避让、手柄和读屏播报仍待补齐或验证。

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
4. 🚧 CommandBar（menus-toolbars，已验证主次命令、动态溢出和独立 AppBar 元素）
5. 🚧 NumberBox（text） / 🚧 AutoSuggestBox（text）
6. ⬜ NavigationView（navigation，或在现有 NavRail 上补齐）
7. ⬜ TabView（navigation，在 TabWidget 上补关闭/新建）
8. ⬜ InfoBadge / PipsPager / PersonPicture（status-info / scrolling / media）
9. ⬜ Flyout / MenuFlyout / TeachingTip（dialogs-flyouts）
10. ⬜ TreeView / RatingControl / ProgressRing / HyperlinkButton

## 每个组件的验收标准

- 编译通过且无警告；`cargo test` 通过。
- 浅色软件渲染截图目检，核对布局、配色、悬停、按下和禁用态。
- 至少一处可运行示例（临时插入预览、验证后移除）证明实际渲染正确。
- 遵循本文件顶部约定（分类目录、偶数长度、Palette、SVG 图标）。
