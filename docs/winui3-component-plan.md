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
- 优先读 `.refs/WinUI/` 的官方 XAML、C++ 和 Gallery 示例。资料缺失时再从上游下载，并在本地记录源文件的 Blob SHA。
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
  navigation/       BreadcrumbBar, NavigationView, TabView, Pivot
  scrolling/        PipsPager, AnnotatedScrollBar ...
  shell/            页面壳 / Frame ...
  status-info/      InfoBar, InfoBadge, ProgressRing ...
  text/             NumberBox, AutoSuggestBox, PasswordBox ...
```

> 主窗口使用 `navigation/navigation_view.slint` 和 `navigation/breadcrumb_bar.slint`。

## Slint 1.18.1 已有（可直接用）

`Button, CheckBox, ComboBox, GroupBox, LineEdit, RadioGroup, ListView, StandardListView, ProgressIndicator, Slider, SpinBox, Spinner, TabWidget, Switch, TextEdit, TimePickerPopup, DatePickerPopup, ScrollView, VerticalBox/HorizontalBox/GridBox, Palette`

## 官方分组 × 可行性

图例：✅ 已有可直接用 · 🚧 已开始复刻 · 🟡 部分可用需增强 · 🔨 需复刻 · ⚠️ 不建议/难等价

### Basic input
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| Button | 🚧 | `components/basic-input/button.slint`，支持普通、强调、透明样式和关联 Flyout |
| RepeatButton | 🚧 | `components/basic-input/repeat_button.slint`，支持长按连发、自定义间隔和三种点击模式 |
| CheckBox | 🚧 | `components/basic-input/check_box.slint`，支持双态、三态、标签布局和状态回调 |
| ComboBox | ✅ | |
| Slider | ✅ | |
| RadioButton | 🟡 | `RadioGroup` 有；便捷分组 `RadioButtons` 待封装 |
| ToggleSwitch | 🟡 | `Switch` 有，样式不完全同 |
| ToggleButton | 🚧 | `components/basic-input/toggle_button.slint`，支持双态、三态、状态回调和自定义内容 |
| DropDownButton / SplitButton / ToggleSplitButton | 🚧 | `components/basic-input/flyout_button.slint`，可接入 Flyout 或 MenuFlyout，支持键鼠、禁用态和选中状态。 |
| HyperlinkButton | 🚧 | `components/basic-input/hyperlink_button.slint`，支持文本、自定义内容、点击回调和 URI 导航 |
| RatingControl | 🚧 | `components/basic-input/rating_control.slint`，支持分数、未评占位、清除、只读及自定义图标 |
| ColorPicker | ⚠️ | 复杂，暂不做 |

运行 `cargo run -p lumatune --example flyout_buttons` 可试用三个按钮，附加 `-- --dark` 切换至深色。已验证鼠标点击、Enter 与 Space、分裂按钮的 F4 与 Alt+↓、弹层关闭后的焦点恢复、禁用态，以及 ToggleSplitButton 用主区域切换状态、用箭头开菜单。Windows UI Automation 已实测展开、调用和切换。AccessKit 通过独立子节点提供分裂按钮的主命令。读屏、原生收起动作、图标动效和触屏仍待验收。[DropDownButton 模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/DropDownButton/DropDownButton.xaml) · [SplitButton 模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/SplitButton/SplitButton.xaml) · [SplitButton 键盘实现](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/SplitButton/SplitButton.cpp)

运行 `cargo run -p lumatune --example rating_control` 可试用占位评分、清除、整数键盘步进和自定义图像。非负的实际评分限制在 1 到最高分之间，负值表示未评分。占位评分允许 0 到 1 之间的小数。程序设值保持小数。鼠标悬停只预览，点击提交，允许清除时再次点击同一分数会清除。方向键和 Home / End 可调整评分，RTL 镜像左右方向与分数裁切。

Windows 浅色已实测 16px 星形、8px 间距、占位色、半星裁切、鼠标悬停放大、键盘、修改最高分后重算、RTL、禁用和只读。UI Automation 已验证 Slider 类型、范围设值及只读状态。WinUI 还提供独立的字符串 ValuePattern 和按语言格式化的评分描述，Slint 用范围值和描述表达。手柄交互、原生缩放动画和读屏播报仍待验收。[WinUI 接口定义](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/RatingControl/RatingControl.idl) · [WinUI 自动化实现](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/RatingControl/RatingControlAutomationPeer.cpp)

#### Button

运行 `cargo run -p lumatune --example button` 可试用普通、强调和透明样式、图标内容、禁用、三种点击模式和关联 Flyout。普通文本通过 `text` 提供，自定义内容通过子元素和 `accessible-name` 提供。启用关联 Flyout 时，按钮先发出 `clicked`，再将当前锚点交给 `open-flyout`。

Windows 浅色已对照 WinUI Gallery 的按钮尺寸、填充、文字和默认释放激活。已实测指针、Enter、空格、Tab、拖出取消、按下及悬停模式、UI Automation Invoke，以及 Flyout 锚定和按 Esc 后的焦点恢复。软件渲染的渐变描边使用首色。WinUI 命令对象、加速键、触控和读屏播报仍待补齐。官方模板、行为及 Gallery 示例已保存在 `.refs/WinUI/`。

#### CheckBox

运行 `cargo run -p lumatune --example check_box` 可试用双态、三态、禁用、RTL 和全选汇总。单项通过 `check-state` 提供状态。程序设值会通知对应状态回调。指针、空格和 UI Automation 激活在状态回调之后发出 `clicked`。

已对照 WinUI Gallery 的 20px 指示框、标签及键盘行为，并在 Windows 浅色实测标签点击、悬停与按下、拖出取消、空格释放、Enter 保持状态、双态数字键盘加减、三态循环、禁用、焦点框和 UI Automation Toggle。加减键按 Slint 字符事件识别，主键盘上的同字符也会触发。勾形与横线取自官方动画几何，Slint 通过水平揭示近似勾形动画，并以无障碍描述表达不确定状态。原生不确定枚举、非默认 `ClickMode`、完整图标动画、触控与读屏播报待补齐。官方模板、动画及 Gallery 示例已保存在 `.refs/WinUI/`。

#### RepeatButton

运行 `cargo run -p lumatune --example repeat_button` 可比较 Press、Release、Hover 和长按计数。默认按下即点击，等待 500ms 后每隔 33ms 连发。空格支持长按，Enter 只在首次触发时点击。按住移出暂停、移回继续，释放、失焦或禁用会结束连发。UI Automation Invoke 触发一次点击。

Windows 浅色已对照 WinUI Gallery 的填充、文字、焦点和默认按下时机。已实测鼠标、空格、Enter、移出重返、禁用、焦点切换、缩短延迟及 UI Automation。Release 和 Hover 已在 Slint 示例中验证。软件渲染的渐变描边仍使用首色，触控和读屏播报待验收。官方模板、时序源码和 Gallery 示例已保存在 `.refs/WinUI/`。

#### ToggleButton

运行 `cargo run -p lumatune --example toggle_button` 可试用双态、三态、禁用与自定义内容。`check-state` 接受未选中、选中与不确定状态。程序设值会触发对应的状态回调；鼠标、键盘和 UI Automation 激活还会在状态回调之后触发 `clicked`。程序可在双态模式下设置不确定状态，下一次激活回到未选中。

已与 WinUI Gallery 对照浅色填充、文字、指针和键盘焦点，并实测拖出取消、松键激活、双态和三态转换、禁用及 UI Automation Toggle。Slint 的软件渲染会将渐变边框按首色绘制，无障碍树用描述表达不确定状态。原生 `Indeterminate` 枚举、非默认 `ClickMode` 和读屏播报待补齐。模板、行为实现和 Gallery 示例已保存在 `.refs/WinUI/`。

#### HyperlinkButton

运行 `cargo run -p lumatune --example hyperlink_button` 可试用 URI 导航、纯点击回调、禁用和下划线。附加 `-- <URI>` 可替换示例目标。激活先执行 `clicked`，再读取当前 `navigate-uri` 并交给系统打开。自定义内容可绑定 `foreground` 跟随交互态，并用 `accessible-name` 提供名称。

已用 WinUI Gallery 对照 Windows 浅色正常、悬停、按下、禁用和焦点状态。示例从 Windows 读取三档强调文字色。已实测鼠标、拖出取消、Tab、Enter 与 Space 松键激活、禁用、系统打开回环 HTTP 链接，以及 UI Automation 的调用能力。Slint 1.18.1 以可调用的 Button 角色提供无障碍入口。原生 Hyperlink 角色、非默认 `ClickMode` 和读屏播报仍待补齐。模板、按钮输入、自动化实现及 Gallery 示例已保存在 `.refs/WinUI/`。

### Collections
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| ListView | ✅ | |
| GridView | 🟡 | `ListView`/`for`+wrap 可拼 |
| ItemsRepeater | 🟡 | `for`/`ListView` 覆盖多数 |
| TreeView | 🚧 | `components/collections/tree_view.slint`，支持层级、三态多选、懒加载、拖放重排和视口虚拟化 |
| FlipView | 🔨 | 轮播 |

运行 `cargo run -p lumatune --example tree_view` 可试用树形行、懒加载、单选和三态多选。模型按稳定 ID 与父 ID 组织，条目顺序决定同级顺序。节点展开回调可加载子项。复选框或空格切换多选，条目点击或 Enter 调用该条目。拖放支持同级插入和改父节点，Alt+Shift+上下键可重排同级项。循环改父、拖到自身和重复移动已由算法处理。

Windows 浅色已实测展开折叠、键盘焦点、多选与半选、懒加载、拖放和键盘重排、RTL，以及 10,000 项列表跳到末行。UI Automation 已读取展开和选中状态。Slint 的无障碍树使用 Tree 容器与可选择列表项，独立提供展开和折叠按钮。WinUI 的自定义项模板、连续拖拽自动滚动、悬停展开、系统跨控件拖放、读屏播报与手柄仍待验收。对照模板和 Gallery 示例已存入 `.refs/WinUI/`。[WinUI TreeViewItem 行为](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/TreeView/TreeViewItem.cpp)

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
| TeachingTip | 🚧 | `components/dialogs-flyouts/teaching_tip.slint`，支持锚点、箭头、主图、操作及可延期关闭 |
| Popup | ✅ | Slint `PopupWindow`/基础可用 |

运行 `slint-viewer --style fluent-light examples/winui3/flyout.slint` 可试用布局、外部关闭和溢出滚动。已用 Fluent 浅色软件渲染与深色 FemtoVG 验证定位、首次焦点、Escape、取消外部关闭和双轴滚动。剩余项包括 Acrylic 背景、`FlyoutShowMode`、关闭时向指定元素传递点击和窗口边界交互。当前用 Acrylic 的纯色回退，指针滚动后焦点移到弹层容器。[WinUI 定位源码](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/FlyoutBase_partial.cpp) · [WinUI 呈现模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommonStyles/FlyoutPresenter_themeresources.xaml)

MenuFlyout 对照微软的 [菜单规范](https://learn.microsoft.com/windows/apps/develop/ui/controls/menus)、[呈现模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommonStyles/MenuFlyout_themeresources.xaml) 和 [级联定位源码](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/CascadingMenuHelper.cpp)。运行 `cargo run -p lumatune --example menu_flyout` 可试用菜单。示例支持 `--dark`、`--compact`、`--edge` 和 `--scoped-shortcuts`，通过 Cargo 的 `--` 传入。

- 浅色软件渲染和深色 FemtoVG 已验证方向键循环、跳过禁用项和分隔线、Enter 与 Space 激活、勾选和单选互斥、菜单内快捷键、鼠标、Escape 与焦点恢复。小窗口已验证 Home 和 End 滚到当前项。
- 三级菜单已验证向右展开、窗口右缘向左翻转、悬停延迟、Esc 与左方向键逐级收起，以及选中命令或点外部时关闭整串弹层。子菜单重叠 4 px，悬停延迟暂用 400 ms。
- 分裂菜单项的主命令和子菜单入口独立命中。已验证方向键在两部分间切换、Enter 与 Space 松开时激活、收起子菜单后焦点回到入口、没有子项时入口禁用。Windows UI Automation 已核对两个按钮名称、禁用状态和焦点切换。键盘行为同时对照了 [WinUI 分裂菜单项源码](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/SplitMenuFlyoutItem_Partial.cpp)。
- `ContextFlyoutArea` 接收鼠标右键、菜单键与 Shift+F10。右键从指针坐标打开，键盘操作从控件边缘打开。已验证贴边翻转和关闭后恢复焦点。自行处理右键的子控件可用 `request-at` 转发。[Windows 上下文命令规范](https://learn.microsoft.com/windows/apps/develop/ui/controls/collection-commanding)
- `MenuFlyoutAccelerators` 可挂在窗口内容或仅挂在焦点范围内。已验证菜单关闭时执行命令、菜单打开时调用未展开的子项、禁用项不响应，以及限定范围内外的差异。[Windows 键盘快捷键规范](https://learn.microsoft.com/windows/apps/develop/input/keyboard-accelerators)
- 剩余项包括读屏播报、触屏长按与笔输入、从右到左的布局、跟随系统悬停延迟、多窗口边界和 Acrylic。

运行 `cargo run -p lumatune --example teaching_tip` 可试用带目标与无目标的引导气泡，附加 `-- --edge` 可试用空间不足时的换向。支持 WinUI 的首选及备选对齐、RTL、独立的尾箭头策略、自动或指定主图位置、可滚动正文及上下文相关的关闭按钮。操作回调由宿主决定是否关闭。宿主可在 `closing` 中取消关闭，或调用 `defer-close()` 后用 `complete-close(cancel)` 处理结果。

Windows 浅色已实测定位、尾箭头、主图和正文滚动、操作按钮、关闭取消和延期、外部点击、Escape 与 F6。UI Automation 已验证操作按钮和调用模式。嵌入时由宿主将 F6 和焦点恢复接到相应控件。气泡受宿主窗口约束，瞬态面板使用纯色；原生跨窗口弹层、Acrylic、样式模板替换、读屏播报与手柄仍待验收。对照模板、行为源码和 Gallery 示例已存入 `.refs/WinUI/`。

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
| BreadcrumbBar | 🚧 | 支持宽度折叠、隐藏路径弹层、键盘导航和 RTL，已接入主窗口页眉 |
| NavigationView | 🚧 | 独立 `components/navigation/navigation_view.slint`，支持自适应窗格、目录层级和顶部溢出 |
| TabView（可增删/关闭） | 🚧 | 独立 `components/navigation/tab_view.slint`，支持可组合页签内容、三种宽度、滚动、关闭和排序 |
| Pivot | 🔨 | 可用 `TabWidget` 近似 |
| SelectorBar | 🔨 | |

### BreadcrumbBar

运行 `cargo run -p lumatune --example breadcrumb_bar` 可试用不同窗口宽度、RTL 和按路径返回。空间不足时按文字实际宽度折叠路径前缀。清单从最近的隐藏祖先排到根节点，选择回传其在原始路径中的索引。已在 Windows 浅色窗口实测溢出布局、鼠标、键盘、Escape 和 RTL。通用 `ItemTemplate`、原生 UI Automation 导航地标和 Acrylic 浮出面仍待补齐。[WinUI 模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/Breadcrumb/BreadcrumbBar.xaml) · [WinUI 布局](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/Breadcrumb/BreadcrumbLayout.cpp)

### NavigationView

运行 `cargo run -p lumatune --example navigation_view` 可切换 Auto、Left、LeftCompact、LeftMinimal 和 Top。已在浅色窗口核对展开窗格、两种覆盖窗格、顶部导航及溢出菜单；用实际窗口宽度验证 640/641 与 1007/1008 两组断点。已验证鼠标选中、方向键和空格、父项展开、子项选择、返回请求、Escape 收起、仅触发命令的项目，以及选中溢出项后将其移回顶部。Windows UI Automation 可读到列表项的选中、禁用和展开状态，Expand 动作可展开父项。

覆盖窗格已验证 Tab 和 Shift+Tab 循环、Escape 后返回触发按钮，以及短窗口滚动菜单时页脚保持可见。紧凑项显示提示。附加 `-- --search` 可验证紧凑搜索打开后直接进入输入框、键盘提交建议，以及切换到顶部模式后保留查询。附加 `-- --top --deep-menu` 可验证选中溢出父项后仍能操作子项，选中叶子时再将整组移回顶部。

顶部导航会在首项前显示窗格标题，并在计算溢出时预留标题宽度。附加 `-- --extended-title-bar` 可验证宿主传入的 32px 标题栏预留高度，及关闭自动预留后导航与内容同步上移。全屏或标准系统标题栏由宿主传入零高度。

附加 `-- --slots --search` 可试用 `pane-header`、`pane-custom-content`、`pane-footer`、`page-header` 和 `content-overlay`。窗格的三个自定义区域在顶部与左侧模式间复用同一组控件，顶部溢出计算会为它们保留空间。页面标题区域可容纳可交互控件，`content-overlay` 位于顶部导航下方。浅色已验证插槽位置、窄窗口溢出、鼠标激活，以及覆盖窗格依次经过自定义控件的正反向 Tab 循环。命名插槽使用 Slint 1.18.1 的实验语法，Cargo 示例已启用。直接用 `slint-viewer` 检查示例时需设置 `SLINT_ENABLE_EXPERIMENTAL_FEATURES=1`。

触控、手柄、UI Automation 的 Collapse 和条目激活仍需核对。覆盖窗格使用纯色 Acrylic 回退。LumaTune 主窗口使用 `NavigationView` 的自适应窗格，并在 `page-header` 插槽中显示路径。[Microsoft NavigationView 说明](https://learn.microsoft.com/windows/apps/develop/ui/controls/navigationview) · [WinUI 模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/NavigationView/NavigationView.xaml)

### TabView

运行 `cargo run -p lumatune --example tab_view` 可切换 Equal、Size to content 和 Compact。附加 `-- --narrow` 可直接查看溢出，附加 `-- --slots` 可试用页签条两端的自定义内容。宿主为各页签分配唯一的稳定 ID，初始化 `TabViewBehavior`，根据 `selected-id` 组织内容，并在新增、关闭或重排请求后更新模型。示例为每个 ID 保存单独的编辑内容。

Windows 浅色已验证三种宽度、窗口缩放、滚动按钮和滚轮、悬停关闭、中键关闭、选中项关闭后的回退、鼠标和 Alt+Shift+方向键重排、拖出页签条的回调、方向键跳过禁用项、Ctrl+Tab / Ctrl+Shift+Tab、Ctrl+F4、首尾插槽和 RTL。Windows UI Automation 已确认页签和按钮类型、选中状态和禁用状态。

待补齐原生跨窗口拖放和撕出、页签头模板、标签和关闭提示、触控与手柄。内容面使用纯色回退。悬停关闭模式为隐藏的关闭按钮保留布局空间，Equal 模式关闭后会立即重排。对照文件存放在 `.refs/WinUI/microsoft-ui-xaml/controls/dev/TabView/` 和 `.refs/WinUI/WinUI-Gallery/WinUIGallery/Samples/TabView/`。

### Status & info
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| InfoBar（可关提示条） | 🚧 | `components/status-info/info_bar.slint`，支持宽窄布局、自定义内容和可区分来源的可取消关闭 |
| InfoBadge（角标） | 🚧 | `components/status-info/info_badge.slint`，支持点、计数、字形与图像，并可附在导航条目上 |
| ProgressBar | 🚧 | `components/status-info/progress_bar.slint`，支持确定和不确定模式、暂停与错误 |
| ProgressRing（环形） | 🚧 | `components/status-info/progress_ring.slint`，支持确定与不确定模式、范围、配色和停用 |
| ToolTip | 🟡 | 基础可用，样式弱 |

运行 `cargo run -p lumatune --example info_badge` 可试用强调色和五种状态色、4×4 点状徽标、16px 高的计数、状态字形和自定义图像。非负 `value` 优先显示数字，负值使用图标或点状状态。自定义图像可保留原色。导航条目通过 `badge` 提供可见性、内容和状态描述，顶部布局会按徽标宽度重新计算。

Windows 浅色已验证六组样式、零值与最大 32 位整数、计数优先级、显隐切换、展开和紧凑窗格、顶部导航、RTL，以及徽标与导航点击的共存。独立徽标可向辅助技术提供非焦点文本，嵌入导航时则作为该条目的状态描述。Slint 会把小于 -1 的值按无数字状态渲染，WinUI 对这类输入抛出异常。Windows 状态字形依赖 Segoe Fluent Icons，读屏播报尚待验收。官方模板及示例已存入 `.refs/WinUI/`。

#### ProgressBar

运行 `cargo run -p lumatune --example progress_bar` 可试用确定进度、不确定进度、暂停、错误、RTL 和内边距。`value` 接收原始进度，`effective-value` 提供范围内的结果。范围放宽后会恢复原始请求值；最小值超过最大值时，最大值随之上调。错误优先于暂停。

Windows 浅色已与 WinUI Gallery 对照，并实测 1px 轨道、六种状态、两段扫描、宽窄尺寸、内边距、RTL 和范围恢复。UI Automation 在确定模式报告只读范围值，在不确定模式报告忙碌、暂停或错误状态。按偶数尺寸约定，默认指示条由 3px 取整到 4px。确定进度的条宽立即更新，原生重新定位和恢复运行的过渡仍有差异。原生模板设置和读屏播报待补齐。模板、行为源码及 Gallery 示例已存入 `.refs/WinUI/`。

#### ProgressRing

运行 `cargo run -p lumatune --example progress_ring` 可切换确定与不确定模式、透明与浅灰轨道、动画和活动状态，并试用 16/32/60px 三种尺寸。范围边界交叉时，刚设置的边界同步收拢另一端，进度值限制在区间内。两端相等时显示空进度。

Windows 浅色已实测尺寸、轨道、升降动画、活动状态和自定义范围。确定样式已与 WinUI Gallery 实例对照，圆弧从 12 点方向顺时针增长。UI Automation 在活动时提供进度条角色，并在确定模式提供只读范围值。Slint 使用矢量圆弧并按项目约定取偶数尺寸。WinUI 的自定义 `IAnimatedVisualSource` 和读屏播报仍待补齐。模板、动画源码、自动化实现及 Gallery 示例已保存在 `.refs/WinUI/`。

### Scrolling
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| ScrollView / ScrollViewer | ✅ | `ScrollView` |
| PipsPager | 🚧 | `components/scrolling/pips_pager.slint`，支持有限和无限分页、横竖方向、循环与按需显示前后按钮 |
| AnnotatedScrollBar | 🔨 | |
| PagerControl | 🔨 | |
| SemanticZoom | 🔨 | |

运行 `cargo run -p lumatune --example pips_pager` 可切换横向、纵向、RTL、有限或无限页数，以及始终显示、悬停或键盘聚焦时显示、折叠三种箭头策略。有限页数可循环翻页。当前页会在可见范围内居中，靠近首尾时对齐边缘。方向键移动焦点，Enter、空格与鼠标点击提交选择。超出范围的有限页索引会归入可用范围。

Windows 浅色已实测分页点尺寸、首尾隐藏、选中点移动、鼠标、键盘、循环、无限分页、减少页数时的回退、空页和禁用。UI Automation 已读到分页点的名称和选中态。Slint 使用列表和可选择条目表达分页状态，WinUI 使用 Menu 自动化角色与选择提供程序。WinUI 样式模板替换、读屏播报和手柄仍待验收。[WinUI 接口定义](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/PipsPager/PipsPager.idl) · [自动化实现](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/PipsPager/PipsPagerAutomationPeer.cpp)

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
| PersonPicture（头像） | 🚧 | `components/media/person_picture.slint`，支持照片、首字母、群组、联系人数据及徽标 |
| AnimatedVisualPlayer（Lottie） | ⚠️ | 依赖 Lottie 运行时 |
| WebView2 / MediaPlayerElement | ⚠️ | 宿主能力，难等价 |

运行 `cargo run -p lumatune --example person_picture` 可试用照片、自动和手动首字母、人员和群组占位符，以及数字、字形和图片徽标。组件接收宿主提供的联系人姓名和各尺寸图片，按 WinUI 顺序回退。照片居中裁切并保留透明圆角。徽标超过 99 时显示 `99+`，无障碍名称保留真实数值。

Windows 浅色已实测照片优先级、联系人小图、姓名和群组回退、圆形裁切、徽标种类与优先级、RTL 和 48/96/144 像素三档。Windows UI Automation 验证了带状态的非焦点文本名称。Windows 联系人对象和异步头像流由宿主提供，目前组件接受已解析的联系人数据。系统读屏和 WinUI 样式替换仍待验收。对照模板和 Gallery 示例已存入 `.refs/WinUI/`。[WinUI 接口定义](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/PersonPicture/PersonPicture.idl) · [首字母实现](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/PersonPicture/InitialsGenerator.cpp)

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
6. 🚧 NavigationView（`navigation/navigation_view.slint`，已验收主要窗格和导航交互，待补宿主插槽及原生无障碍差异）
7. 🚧 TabView（`navigation/tab_view.slint`，已验证桌面常用交互，待补原生跨窗口拖放、模板和提示）
8. 🚧 InfoBadge（已验收浅色呈现、动态值及导航用法） / 🚧 PipsPager（已验收浅色和常用交互） / 🚧 PersonPicture（浅色已验收，待接入 Windows 联系人）
9. 🚧 Flyout / MenuFlyout / TeachingTip（已实测引导气泡的定位和关闭流程）
10. 🚧 TreeView（已验收浅色及常用输入） / 🚧 RatingControl（浅色及常用操作已验收） / 🚧 ProgressRing（已验收浅色、动画及范围） / 🚧 HyperlinkButton（浅色、常用激活方式及 URI 导航已验收）

## 每个组件的验收标准

- 编译通过且无警告；`cargo test` 通过。
- 浅色软件渲染截图目检，核对布局、配色、悬停、按下和禁用态。
- 至少一处可运行示例（临时插入预览、验证后移除）证明实际渲染正确。
- 遵循本文件顶部约定（分类目录、偶数长度、Palette、SVG 图标）。
