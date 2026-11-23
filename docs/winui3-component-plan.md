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
- 实现前按控件分组批量缓存官方 XAML、C++ 和 Gallery 示例到 `.refs/WinUI/`。直接用本地文件核对模板、行为和主题。缓存缺项时从固定的上游提交补齐，并在 `sources.json` 记录提交和 Git Blob SHA。
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
| ComboBox | 🚧 | `components/basic-input/combo_box.slint`，支持占位、暂存选择、可编辑提交和选项图标及字体 |
| Slider | 🚧 | `components/basic-input/slider.slint`，支持方向、刻度、吸附、数值提示和键盘 |
| RadioButton | 🚧 | `components/basic-input/radio_button.slint`，支持独立单选、文字和自定义内容 |
| RadioButtons | 🚧 | `components/basic-input/radio_buttons.slint`，支持互斥、按列排列、自适应列数和键盘导航 |
| ToggleSwitch | 🚧 | `components/basic-input/toggle_switch.slint`，支持拖动切换、标题、状态文字和自定义内容 |
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

#### Slider

运行 `cargo run -p lumatune --example slider` 可试用横向和纵向滑块、RTL、反向、刻度吸附、小数步进及禁用。拖动时拇指跟随连续位置，公开值按步长或刻度吸附。程序设值只钳制范围，键盘方向键使用 `small-change`，吸附到刻度时改用 `tick-frequency`。LumaTune 现有业务滑块继续使用 Slint 原生连续调节。

Windows 浅色已对照 WinUI Gallery 的 4px 轨道、22px 拇指外圈、双侧刻度、焦点框和按下数值提示。已实测轨道点击、拖动和释放、四个方向键、Home / End、非整倍数端点、RTL、反向、纵向刻度、程序设值、小数、禁用及 UI Automation RangeValue。提示在父容器内绘制，边缘避让、独立的 UI Automation 大步长、手柄和读屏播报仍待补。官方模板、步进算法、刻度算法及 Gallery 示例已保存在 `.refs/WinUI/`。

#### ComboBox

运行 `cargo run -p lumatune --example combo_box` 可试用颜色占位、选项字体和可编辑字号。接入组件时先注册 `ComboBoxBehavior`，并传入宿主视口尺寸以启用可编辑弹层的边缘避让和外部点击关闭。默认在确认后更新选中项，也可以即时更新。可编辑输入支持前缀补全和自定义值，`text-submitted` 返回 true 时回到当前已提交值。

Windows 浅色已与 WinUI Gallery 对照标题、32px 字段、占位文字、图标和弹层。已实测 F4、Alt＋方向键、Space、方向键暂存、Enter 提交、Esc 恢复、外部点击关闭、前缀查找、禁用项、RTL、Editable 文本校验，以及 UI Automation 的 ExpandCollapse 状态和 SelectionItem 选择。选项目前可指定图片与字体，任意选项模板、Acrylic 材质、触控和读屏待补齐。可编辑弹层在宿主内绘制，跨窗定位及输入法候选窗避让待补。官方模板、选择行为和 Gallery 示例已保存在 `.refs/WinUI/`。

#### RadioButton 和 RadioButtons

运行 `cargo run -p lumatune --example radio_buttons` 可试用独立按钮、禁用项、多列、窄宽换行和 RTL。接入单选组时注册 `RadioButtonsBehavior`，给每个选项提供非空且唯一的 `id`。通过 `selected-index` 或 `select-id()` 设置选中项，通过 `selected-item` 读取选项，通过 `selection-changed` 接收新旧索引。替换选项模型时，本实现按 ID 保留选中项，删除选中项后清除选择。WinUI 替换整个 ItemsSource 时清空选择。

Windows 浅色已用 WinUI Gallery 对照圆形指示器、正常和按下填充、20px 指示器与 32px 控件高度。已实测标签点击、拖出取消、Space、Enter、Tab 回焦、方向键跳过禁用项、Ctrl＋方向键只移动焦点、RTL、长标签换行、重排和删除，以及 UI Automation 的 SelectionItem 和选择容器。列间距将官方的 7px 按项目规则取为 8px。独立按钮的 WinUI GroupName、单选组选项模板、圆点渐变描边、触控和读屏播报仍待补齐。官方模板、布局算法和 Gallery 示例已保存在 `.refs/WinUI/`。

#### RepeatButton

运行 `cargo run -p lumatune --example repeat_button` 可比较 Press、Release、Hover 和长按计数。默认按下即点击，等待 500ms 后每隔 33ms 连发。空格支持长按，Enter 只在首次触发时点击。按住移出暂停、移回继续，释放、失焦或禁用会结束连发。UI Automation Invoke 触发一次点击。

Windows 浅色已对照 WinUI Gallery 的填充、文字、焦点和默认按下时机。已实测鼠标、空格、Enter、移出重返、禁用、焦点切换、缩短延迟及 UI Automation。Release 和 Hover 已在 Slint 示例中验证。软件渲染的渐变描边仍使用首色，触控和读屏播报待验收。官方模板、时序源码和 Gallery 示例已保存在 `.refs/WinUI/`。

#### ToggleSwitch

运行 `cargo run -p lumatune --example toggle_switch` 可试用默认开关、自定义状态文字、禁用和 RTL。`is-on` 接受双向绑定，鼠标、键盘、UI Automation 或程序修改状态时都会触发 `toggled`。默认文字可本地化，UI Automation 名称包含标题与自定义状态文字，也可通过 `accessible-name` 指定。

Windows 浅色已对照 WinUI Gallery 的 40×20px 轨道、12px 文字间距、40px 操作区、标题、颜色、悬停、按下和焦点框。已实测标题和尾部留白点击不触发、状态文字点击、Space 松键切换、Enter 与方向键保持状态、水平拖动到轨道中点、垂直移动取消点击、RTL 拖动、禁用以及 UI Automation Toggle。鼠标点击允许 2px 位移，按下时的滑块宽度从原版 17px 取整为 18px。滑块回位采用 Slint 位置动画，时序与系统动画有差异。独立的标题与状态模板、系统触控手势、手柄和读屏播报仍待补齐。官方模板、行为和 Gallery 示例已保存在 `.refs/WinUI/`。

#### ToggleButton

运行 `cargo run -p lumatune --example toggle_button` 可试用双态、三态、禁用与自定义内容。`check-state` 接受未选中、选中与不确定状态。程序设值会触发对应的状态回调；鼠标、键盘和 UI Automation 激活还会在状态回调之后触发 `clicked`。程序可在双态模式下设置不确定状态，下一次激活回到未选中。

已与 WinUI Gallery 对照浅色填充、文字、指针和键盘焦点，并实测拖出取消、松键激活、双态和三态转换、禁用及 UI Automation Toggle。Slint 的软件渲染会将渐变边框按首色绘制，无障碍树用描述表达不确定状态。原生 `Indeterminate` 枚举、非默认 `ClickMode` 和读屏播报待补齐。模板、行为实现和 Gallery 示例已保存在 `.refs/WinUI/`。

#### HyperlinkButton

运行 `cargo run -p lumatune --example hyperlink_button` 可试用 URI 导航、纯点击回调、禁用和下划线。附加 `-- <URI>` 可替换示例目标。激活先执行 `clicked`，再读取当前 `navigate-uri` 并交给系统打开。自定义内容可绑定 `foreground` 跟随交互态，并用 `accessible-name` 提供名称。

已用 WinUI Gallery 对照 Windows 浅色正常、悬停、按下、禁用和焦点状态。示例从 Windows 读取三档强调文字色。已实测鼠标、拖出取消、Tab、Enter 与 Space 松键激活、禁用、系统打开回环 HTTP 链接，以及 UI Automation 的调用能力。Slint 1.18.1 以可调用的 Button 角色提供无障碍入口。原生 Hyperlink 角色、非默认 `ClickMode` 和读屏播报仍待补齐。模板、按钮输入、自动化实现及 Gallery 示例已保存在 `.refs/WinUI/`。

### Collections
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| ListView | 🚧 | `components/collections/list_view.slint`、`list_view_item.slint`，支持 WinUI 条目模板、四种选择模式、分组、吸顶、同列表拖动和虚拟化 |
| GridView | 🚧 | `components/collections/grid_view.slint`，支持自适应行列、四种选择模式、拖动重排和视口虚拟化 |
| ItemsRepeater | 🚧 | `components/collections/items_repeater.slint`，提供 StackLayout、UniformGridLayout、FlowLayout、LinedFlowLayout 和自定义位置，支持宿主模板和视口虚拟化 |
| ItemsView / ItemContainer | 🚧 | `components/collections/items_view.slint`、`item_container.slint`，支持三种 Gallery 布局、四种选择模式、独立条目调用和自定义内容 |
| TreeView | 🚧 | `components/collections/tree_view.slint`，支持层级、三态多选、懒加载、拖放重排和视口虚拟化 |
| FlipView | 🚧 | `components/collections/flip_view.slint`，支持横纵分页、选择、鼠标、键盘和触控翻页 |

运行 `cargo run -p lumatune --example tree_view` 可试用树形行、懒加载、单选和三态多选。模型按稳定 ID 与父 ID 组织，条目顺序决定同级顺序。节点展开回调可加载子项。复选框或空格切换多选，条目点击或 Enter 调用该条目。拖放支持同级插入和改父节点，Alt+Shift+上下键可重排同级项。循环改父、拖到自身和重复移动已由算法处理。

Windows 浅色已实测展开折叠、键盘焦点、多选与半选、懒加载、拖放和键盘重排、RTL，以及 10,000 项列表跳到末行。UI Automation 已读取展开和选中状态。Slint 的无障碍树使用 Tree 容器与可选择列表项，独立提供展开和折叠按钮。WinUI 的自定义项模板、连续拖拽自动滚动、悬停展开、系统跨控件拖放、读屏播报与手柄仍待验收。对照模板和 Gallery 示例已存入 `.refs/WinUI/`。[WinUI TreeViewItem 行为](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/TreeView/TreeViewItem.cpp)

#### ListView

运行 `cargo run -p lumatune --example list_view` 查看基本联系人、双行自定义模板、分组标题和拖动排序。宿主注册 `ListViewBehavior`、`ItemsViewBehavior` 和 `ItemsRepeaterBehavior`，按显示顺序提供非空唯一键、可选分组及条目高度。自定义模板通过 `realized` 使用原始条目索引，以条目公开的 `content-width` 和 `content-height` 排列正文，并绑定行选择和操作回调。`ItemClick` 在本次选区更新前触发，`scroll-into-view` 支持最小滚动和顶端对齐。

原生 Gallery 在 125% 下的基础行高为 50 物理像素，对应模板的 40px。复刻遵循 16/12px 正文内边距、4/2px 底板内收、20px 内联复选框、14px 复选框起点和 28px 正文偏移。单选和扩展选择显示指示条，多选使用可隐藏的复选框。指示条的原生 3px 宽度按项目约定取 4px。

浅色 100% 和 125% 检查了基础与自定义行、点按和键盘、四种选择模式、禁用项、事件顺序、分组吸顶、RTL、同列表拖动和顶端滚动。跳到 10,000 项末行时创建 20 个条目。Windows UI Automation 验证了 List 与 ListItem 角色、单选互斥和条目禁用状态。共用的 WinUI 滚动视口将 RTL 垂直滚动条放在左侧。跨列表拖放、拖动自动滚动、原生阶段式模板绑定、系统滚动位置恢复、完整 UI Automation Invoke 与多项选择动作、读屏和手柄仍待对齐。Gallery 示例、模板与原生行绘制源码已存入 `.refs/WinUI/`。[ListView 行绘制](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/dxaml/xcp/core/core/elements/ListViewBaseItemChrome.cpp) · [Gallery ListView 示例](https://github.com/microsoft/WinUI-Gallery/blob/6de91feccadfbb0a796b71caedead5a2284f730e/WinUIGallery/Samples/ListView/ListViewPage.xaml)

#### GridView

运行 `cargo run -p lumatune --example grid_view` 可试用窗口宽度变化、横纵流向、RTL、选择模式和拖动排序。接入时注册 `GridViewBehavior`，为条目提供非空且唯一的 ID。`selected-ids` 支持双向绑定，选择在重排后按 ID 保留。宿主可启用条目调用和重排回调。

Windows 浅色用本地缓存的 Gallery 原图对照了 190×130px 照片、4px 间距、悬停边线、按下状态和选中的蓝白双层边框。已实测窄宽换列、鼠标和键盘选择、Ctrl 保留选区、Shift 范围、禁用项、无选择模式下的条目调用、拖动重排和 RTL。跳到 10,000 项模型末尾时创建了 10 个格子，UI Automation 能读取末项及其选中状态。

FemtoVG 正常裁切照片圆角，Slint 1.18.1 的软件渲染会保留图片直角。滚动视口边缘会裁切外侧焦点框。通用逐项模板、可变尺寸和分组、跨控件拖放、拖拽自动滚动、原生多选的 UI Automation 声明、读屏和手柄仍待对齐。模板、行为源码、Gallery 页面和照片均已保存在 `.refs/WinUI/`。

#### ItemsRepeater

运行 `cargo run -p lumatune --example items_repeater` 可试用色条、圆形、网格、可变尺寸流式布局和自定义排列。宿主注册 `ItemsRepeaterBehavior`，将自己的数据长度、视口尺寸和内容坐标中的滚动偏移传给组件，在 `for rect in repeater.realized` 中按原始索引绑定 Slint 条目。模板自行提供交互和无障碍语义。提供与数据行对应的非空唯一键，可在滚动或增删导致条目换索引时保留仍在视口缓存内且相对顺序未变的条目状态。

Windows 浅色对照了 Gallery 三条色条的强调色、底色、24px 高度和 8px 间距。官方 425px 色条槽宽按项目规则取 426px。实测增删至空、横纵方向、RTL、窄宽换行、对齐、可变尺寸、按索引跳转和自定义排列。10,000 项纵向列表在首尾各创建 8 个条目。已验证滚动及移除首项后仍在缓存内的开关保留选中态。

条目尺寸由宿主提供，自定义布局通过坐标模型接入。内置布局预计算整批几何，然后按视口及缓存创建元素。WinUI 的逐项测量、元素生命周期事件、元素池、离屏焦点与锚点保持仍待补齐。官方行为源码与 Gallery 示例已存入 `.refs/WinUI/`。

#### ItemsView / ItemContainer

运行 `cargo run -p lumatune --example items_view` 可查看基本列表、可换布局和选择与调用三个场景。宿主注册 `ItemsViewBehavior` 与 `ItemsRepeaterBehavior`，为每行提供非空唯一键和启用状态，在 `for rect in view.realized` 中提供 ItemContainer 及自定义内容。控件公开当前索引、所选键、选择与调用事件、范围选择、反选、视口位置查询和按索引滚动。LinedFlowLayout 根据条目宽高比与统一行高排列，支持项目最小宽度、行与列间距、填充及 RTL。

Windows 浅色对照了 Gallery 基本示例的 220×400px 视口、200×140px 条目和 4px 图片内边距，以及模板的强调色选择边框、内描边、悬停和按下色、20px 多选复选框。按偶数取整约定，3px 外框取 4px，5px 布局间距取 6px。Slint 在 100% 和 125% 实测了鼠标选择、双击和 Enter 调用、Space、Shift/Ctrl 选区、禁用调用、三种布局切换、RTL 及 10,000 项末尾定位。末尾测试创建了 10 个条目。Windows UI Automation 已验证 List、ListItem 和单选互斥。

LinedFlowLayout 的全局行划分优化、晚到的尺寸信息和行锁定，ScrollView 原生滚动物理与外接滚动控制器，原生 Invoke、ScrollItem 和读屏播报，触控和手柄仍待补齐。软件渲染时贴视口的焦点框和多选框可能裁切，照片标题与喜欢数使用本地样例数据。官方模板、选择行为和 Gallery 示例已存入 `.refs/WinUI/`。[WinUI ItemContainer 模板](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/ItemContainer/ItemContainer.xaml) · [Gallery ItemsView 示例](https://github.com/microsoft/WinUI-Gallery/blob/6de91feccadfbb0a796b71caedead5a2284f730e/WinUIGallery/Samples/ItemsView/ItemsViewPage.xaml)

#### FlipView

运行 `cargo run -p lumatune --example flip_view` 可试用图片页和自定义内容。示例优先使用本地缓存的 Gallery 图片，缺少图片时显示页名。接入组件时先注册 `FlipViewBehavior`。选项使用非空且唯一的 ID，重排时按 ID 保持选中项，删除时选择原索引处可用的页。列表为空时索引为 -1。

Windows 浅色已用同一组 Gallery 图片对照 400×270px 页面、16×38px 水平按钮、38×16px 垂直按钮、实心箭头和焦点框。已实测首尾页按钮隐藏、3 秒隐退、停留在按钮上保持显示、方向键、PageUp / PageDown、Home / End、滚轮合并、RTL、键鼠禁用、重排和清空。Windows 触控注入验证了 Slint 横纵滑动。UI Automation 已验证 List 选择与按钮 Invoke，只有当前页进入无障碍树。

按钮内缩把官方的 1px 按项目规则取为 2px。自定义内容使用当前页插槽。Acrylic 材质、原生触控惯性、逐项模板、手柄和读屏仍待对齐。官方模板、行为、Gallery 示例和校验过的图片均已保存在 `.refs/WinUI/`。

### Date & time
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| CalendarDatePicker | 🚧 | `components/date-time/calendar_date_picker.slint`，复用日历面板，支持可空日期、日期格式、键鼠操作和适应视口的弹层 |
| DatePicker | 🚧 | `components/date-time/date_picker.slint`，支持本地化日月年滚轮、可空日期、确认和取消 |
| TimePicker | 🚧 | `components/date-time/time_picker.slint`，支持系统、12 小时和 24 小时制、分钟步进与可空时间 |
| CalendarView | 🚧 | `components/date-time/calendar_view.slint`，支持月、年、年代视图，日期范围、禁选、单选与多选 |

#### CalendarView

运行 `cargo run -p lumatune --example calendar_view` 可试用三层视图、日期范围、禁选日期、RTL、语言与历法。宿主注册 `CalendarViewBehavior`，并用 `lumatune_win::calendar::date_key` 构造日期。选择和边界使用从公元 1 年 1 月 1 日起的日序号，该日为 1。历法和语言改变时，日期值仍指向同一天。`selected-dates-changed` 回传增减项，`date-invoked` 可供日期选择器提交。

Windows 浅色已对照 Gallery 的 302×306px 面板、日期图形、标题和周标题、年视图月份缩写与年代网格。Gallery 对照示例展示五行，组件默认六周。已实测前后翻页、下钻、方向键、PageUp/PageDown、Home/End、Ctrl+上下键、选中与取消、禁选和范围截断。中英文切换、Hebrew、Hijri 和 Japanese 标题已在示例中验证。UI Automation 已读取列表的选中日期。Gregorian 闰日和后三种历法的日期计算有回归测试。对照模板及行为源码保存在 `.refs/WinUI/`。

当前滚轮按周期分页。连续触控滚动、日期密度标记、逐项容器生命周期、完整的历法换代边界及 WinUI 网格无障碍模式仍待补齐。

#### CalendarDatePicker

运行 `cargo run -p lumatune --example calendar_date_picker` 可试用日期回填、清空、范围约束、短日期与自定义格式、RTL 和弹层翻转。宿主注册 `CalendarViewBehavior`。`date` 为日序号，0 表示空。`date-format` 为空时用系统短日期，显式格式按控件的语言和历法交给 Windows 格式化。`date-changed` 返回旧值与新值。`set-display-date` 可单独移动日历视图，传入 `popup-viewport` 可让弹层在宿主可用区域内居中并翻转。

Windows 浅色已对照 Gallery 的标题、32px 字段、日期文字和 302×354px 日历面板。已实测打开、Escape、点击外部关闭、键盘选日、选择回填和再次点选同一天。再次点选会清空日期并保留弹层。下方和上方打开时均与字段留 4px 间距。Windows UI Automation 已验证 Button、Invoke 和只读 Value。模板、事件逻辑和原生自动化实现已保存在 `.refs/WinUI/`。

弹层在宿主窗口内定位。WinUI 原生弹层跨窗口边界的行为、触控轻触关闭遮罩、自定义标题与日历样式仍待补齐。

#### DatePicker

运行 `cargo run -p lumatune --example date_picker` 可切换字段顺序、年字段显示、日期格式、范围和 RTL。宿主注册 `DatePickerBehavior`。`selected-date` 使用日序号，0 表示空。`min-year` 和 `max-year` 接受所在年份中的任意日期，范围包含首尾整年。滚轮独立修改暂选日期，确认后才更新字段，Escape、取消和点击外部可放弃暂选。

Windows 浅色按 Gallery 字段和官方弹层模板核对了 296px 宽度、三列分隔、40px 滚轮行和确认与取消按钮。已实测键盘切换列、鼠标选项、滚轮步进、闰日切年约束、范围、月日在栏内循环、英美字段顺序、自定义日格式、隐藏年份和 RTL。原生惯性滚动、跨 HWND 弹出、完整的滚动 UI Automation 模式和换代边界仍待补齐。参考文件保存在 `.refs/WinUI/`。

#### TimePicker

运行 `cargo run -p lumatune --example time_picker` 可试用 12／24 小时制、清空、分钟步进、确认与取消。宿主注册 `TimePickerBehavior`。`selected-time` 使用从午夜起算的分钟数，`-1` 表示空。`minute-increment` 为 1–59 时向下取到最近档位，0 时分钟固定为 `00`。时制和地区格式由 Windows 提供。

窗口字段与官方 242px 最小宽度对照，弹层复用 `DatePicker` 的 40px 滚轮行、选中条、焦点和确认按钮。Windows 浅色已实测 AM/PM、午夜与正午、24 小时循环、1／15／0 分钟步进、RTL、禁用、键鼠确认及 Escape 和取消。共享弹层后复验了 `DatePicker` 的原有操作。原生惯性滚动、跨 HWND 弹出和完整滚动 UI Automation 模式仍待对齐。对照模板与 Gallery 示例保存在 `.refs/WinUI/`。

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
| Grid / StackPanel | ✅ | `GridBox/HorizontalBox/VerticalBox` |
| Border | 🚧 | 逐边厚度、逐角圆角、内容内边距与两种背景边界 |
| Canvas | 🚧 | 绝对布局、原始子项尺寸、动态层级、RTL 及显式裁切 |
| RelativePanel | 🚧 | 面板与兄弟锚点、约束优先级、双边拉伸、自动首选尺寸和无效引用检测 |
| Viewbox | 🚧 | 整体变换、四种缩放模式、方向限制、对齐及裁切 |
| Expander | 🚧 | `components/layout/expander.slint`，支持标题插槽、内容对齐、禁用和双向展开 |
| SplitView | 🚧 | `components/layout/split_view.slint`，含四种窗格模式、左右放置、内容插槽与可取消的轻触关闭 |
| WrapPanel | 🚧 | 双向换行、内边距、行和项间距、末项拉伸、折叠与 RTL |
| VariableSizedWrapGrid | 🚧 | 跨行跨列、槽位尺寸推断、行列上限、边界裁切和对齐 |

#### Expander

运行 `cargo run -p lumatune --example expander` 可试用 Gallery 的文本标题、居中标题、可编辑内容、RTL 及向上和向下展开。通过 `header-content` 提供标题，通过默认插槽提供正文。常规正文居中，宿主可指定对齐和内边距。折叠保留内容实例，展开后恢复原有输入。向上展开时可由宿主固定标题的底部锚点。

已对照原生 WinUI Gallery 的标题、正文位置和展开方向。浅色 100% 和 125% 下检查了四种箭头朝向、圆角接缝、对齐、RTL、键鼠操作、禁用和程序展开。组件禁用时拦截其范围内的指针和激活键，正文控件将 `enabled` 绑定到 `content-enabled` 后同时停用焦点、编辑和无障碍操作。布局立即切换，箭头保留旋转过渡。

Windows UI Automation 已验证展开、展开状态和绑定后的子控件失效。Slint 1.18.1 未向组件转发原生 `ExpandCollapse.Collapse`，也未向任意 Slint 子树自动继承禁用状态。折叠的辅助技术播报仍待验收。[官方模板](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/Expander/Expander.xaml) · [Gallery 示例](https://github.com/microsoft/WinUI-Gallery/blob/6de91feccadfbb0a796b71caedead5a2284f730e/WinUIGallery/Samples/Expander/ExpanderPage.xaml)

#### SplitView

运行 `cargo run -p lumatune --example split_view` 可切换四种模式、窗格方向和遮罩。通过 `pane`、`content` 命名插槽承载内容。内联模式让正文为窗格腾出空间，覆盖模式将窗格叠在正文上，紧凑覆盖模式仍保留紧凑栏。紧凑态只裁剪窗格视口，内部按展开宽度布局。窗格外点击、Escape 和大小变化触发的关闭可以在 `pane-closing` 中取消，显式关闭会发送同一通知并完成关闭。

已对照本地官方模板和 Windows 浅色 Gallery，检查内联示例的 400 × 300 区域、256 宽窗格、正文换行及面板底色。已实测四种模式、左右放置和 RTL、Tab 循环、Escape 与焦点恢复、遮罩拦截、关闭取消及 UI Automation 的遮罩调用。嵌入时由宿主接入 `SplitViewBehavior.traverse-focus` 和 `restore-focus`。组件边界外的关闭请求可由宿主转给 `dismiss-pane()`。原生覆盖层的整窗命中、自动测量窗格宽度、窗格 UIA Window 角色和示例图标字形仍有差异。

#### WrapPanel 与 VariableSizedWrapGrid

运行 `cargo run -p lumatune --example wrap_layout` 查看 Gallery 的十项换行、末项拉伸和四色跨格示例。宿主注册 `WrapLayoutBehavior`，按子项顺序提供测量尺寸、跨度和折叠状态，并按 `placements` 放置插槽中的 Slint 子元素。RTL 镜像横向槽位。可用尺寸为负时，对应轴视为无上限。

WrapPanel 共享同一行或列的交叉轴尺寸，跳过折叠项。如果源模型最后一项折叠，前一项保留原尺寸。VariableSizedWrapGrid 从当前位置向前搜索连续空格，保留前面留下的空位，沿增长轴裁切跨出边界的槽。显式单元格尺寸省略时使用首项的测量尺寸。出界未摆放的条目保留原索引，其槽位尺寸为零。

已对照本地 WinUI 代码和 Gallery XAML。Windows 浅色 100% 和 125% 用 Slint MCP 验证了行列位置、横纵切换、缩窄重排、间距、末项隐藏与拉伸、四色尺寸与颜色、跨格上限、对齐、RTL 及重排后保留编辑内容。Rust 回归测试覆盖跨格顺序、有限空间裁切和末项折叠。原生 Gallery 的基础 WrapPanel 已核对换行和相对位置。示例按偶数度量将常规卡片高度取为 38，仍需完成原生截图逐像素对照。Slint 插槽子项的测量与放置由宿主绑定，原生 XAML 的自动 Measure/Arrange 与无障碍树映射仍待验证。[WrapPanel 源码](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/WrapPanel/WrapPanel.cpp) · [VariableSizedWrapGrid 源码](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/dxaml/xcp/dxaml/lib/VariableSizedWrapGrid_Partial.cpp)

#### RelativePanel

运行 `cargo run -p lumatune --example relative_panel` 查看 Gallery 四色示例，以及可编辑的双边约束、内边距、RTL 和失效引用示例。宿主注册 `RelativePanelBehavior`，为子元素提供期望尺寸、外边距和两轴约束，按模型索引连接布局结果。约束可通过 ID 引用位置在前或在后的兄弟。两轴共同检测循环，连优先级较低而被覆盖的引用也参与检测。无效图通过 `layout-issue` 和 `issue-target` 报告。宿主可以在修正后恢复布局并保留原子元素。

浅色 100% 和 125% 的 Slint 运行验证覆盖 Gallery 四色色块的颜色、50 × 50 尺寸、8 像素间隔和对齐位置，并检查了首选高度、变化宽度、双边拉伸、内边距、RTL、折叠、无效约束以及输入保留。回归测试对照原生优先级和居中链的预期结果。子项的测量尺寸由宿主提供。坐标按项目的偶数规则取整，原生截图对照、自动子元素测量、逐侧边框和原生布局裁切仍待验收。[WinUI 布局算法](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/dxaml/xcp/components/relativepanel/lib/RPGraph.cpp) · [Gallery 示例](https://github.com/microsoft/WinUI-Gallery/blob/6de91feccadfbb0a796b71caedead5a2284f730e/WinUIGallery/Samples/RelativePanel/RelativePanelPage.xaml)

#### Viewbox

运行 `cargo run -p lumatune --example viewbox` 查看 Gallery 色条、`Slices.png`、文字和可交互子项。组件根据插槽的首选尺寸计算比例，宿主可显式覆盖原始尺寸。支持 None、Fill、Uniform、UniformToFill 及 Both、UpOnly、DownOnly。单轴有上限时沿用该轴比例，双轴无上限时保持原始大小。超出视口时裁切，默认 Stretch 对齐在溢出轴锚定起点，显式 Center 则居中裁切。

Windows 浅色 FemtoVG 100% 和 125% 已检查缩放后的颜色、原始图片、比例、对齐、剪裁像素、放大后的指针和键盘激活，以及裁切范围外的点按。无窗口回归测试覆盖无上限轴、空尺寸和方向钳制。软件渲染示例以显式尺寸接入 `scale-x` 和 `scale-y`，在 125% 验证了几何、图像裁切和交互。Slint 软件渲染器的任意子树变换不可用，示例的按钮文字也保留正常字号。Gallery 边框按偶数规则由 15 取为 16 像素。原生截图逐像素对照和变换后 Windows UI Automation 边界仍待验收。[WinUI Viewbox 源码](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/dxaml/xcp/core/core/elements/Viewbox.cpp) · [Gallery 示例](https://github.com/microsoft/WinUI-Gallery/blob/6de91feccadfbb0a796b71caedead5a2284f730e/WinUIGallery/Samples/Viewbox/ViewboxPage.xaml)

#### Border 与 Canvas

运行 `cargo run -p lumatune --example border_canvas` 查看 Gallery 的金色边框、白底文字和 140 × 140 画布上的四个色块。Border 将各边厚度和内边距加入首选尺寸，按内边界或外边界绘制背景。非对称圆角由相邻两边的厚度分别调整，相遇时按比例收拢。方角边框使用矩形。Canvas 的首选尺寸为零，子项保持自身尺寸并按 `left` 和 `top` 放置。直接在子项实例上绑定 Slint 的 `z`，同层级的后声明项优先绘制和命中。画布默认允许溢出，宿主可以开启 `clip`。

浅色 FemtoVG 和软件渲染器均在 100% 和 125% 下实测边框颜色、零厚度、内外背景、狭窄圆角、内容位置、画布重叠层级、键鼠命中、RTL 以及溢出裁切。Gallery 文字的纵向边距按偶数约定由 5 取为 6 像素。原生逐像素截图对照、Border 的圆角命中和 Windows UI Automation 仍待验收。[WinUI Border 源码](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/dxaml/xcp/core/core/elements/Border.cpp) · [WinUI Canvas 源码](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/dxaml/xcp/core/core/elements/canvas.cpp) · [Gallery Border](https://github.com/microsoft/WinUI-Gallery/blob/6de91feccadfbb0a796b71caedead5a2284f730e/WinUIGallery/Samples/Border/BorderPage.xaml) · [Gallery Canvas](https://github.com/microsoft/WinUI-Gallery/blob/6de91feccadfbb0a796b71caedead5a2284f730e/WinUIGallery/Samples/Canvas/CanvasPage.xaml)

### Navigation
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| BreadcrumbBar | 🚧 | 支持宽度折叠、隐藏路径弹层、键盘导航和 RTL，已接入主窗口页眉 |
| NavigationView | 🚧 | 独立 `components/navigation/navigation_view.slint`，支持自适应窗格、目录层级和顶部溢出 |
| TabView（可增删/关闭） | 🚧 | 独立 `components/navigation/tab_view.slint`，支持可组合页签内容、三种宽度、滚动、关闭和排序 |
| Pivot | 🚧 | `components/navigation/pivot.slint`，支持标题、侧边插槽、单选页签和触摸切页 |
| SelectorBar | 🚧 | `components/navigation/selector_bar.slint`，支持文字与图标、单选、独立焦点和横向滚动 |

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

### Pivot

运行 `cargo run -p lumatune --example pivot` 可试用 Gallery 的邮件页签示例及窄栏、RTL、锁定和动态条目。宿主注册 `PivotBehavior` 并提供唯一的条目 ID，用 `selected-id` 切页，按有效的 `selected-index` 渲染正文。标题和两侧附加内容可用命名插槽替换。

Windows 浅色已对照 48px 标题栏、标题与正文位置、24px 页签文字及选中线。已按原生宽栏实测首尾边界、方向键、Home / End、Ctrl+Tab 和 Ctrl+PageDown。Slint 示例另外验证了窄栏指针按钮、移除与禁用项、RTL、锁定、内容区快捷键、UI Automation SelectionItem 和 Windows 触摸输入。选中线按偶数尺寸从 3px 取整为 4px，字体测量使部分页签略窄。窄栏目前按固定顺序滚动标题，待核对原生动态标题布局与触摸惯性。页签头模板、页面加载和卸载事件及读屏仍待补齐。模板和 Gallery 示例已存入 `.refs/WinUI/`。

### SelectorBar

运行 `cargo run -p lumatune --example selector_bar` 可试用 Gallery 的文件筛选、页面切换和颜色集合示例。宿主注册 `SelectorBarBehavior`，用唯一 ID 标识条目。初始可无选中项，焦点首次进入时自动选中获得焦点的可用项。方向键直接切换，Ctrl+方向键只移动焦点，Enter 或空格提交。Tab 前后重新进入时回到上次获得焦点的可用项。

已在 Windows 浅色对照 Gallery 的 40px 条目高度、文字和图标位置、焦点框及选中条颜色。实测了左右键与首尾键、焦点回访、程序和无障碍选中、禁用、重排、删除、RTL、窄栏滚轮和触摸拖动。拖动滚动不改选，轻点可选择露出的条目。选中条高度按偶数约定由 3px 取 4px。图标使用仓库中的 Fluent SVG，原生 `ItemContainer.Child` 自定义内容、完整 UI Automation 滚动模式和读屏仍待补齐。Gallery 页面示例即时切换。官方模板、行为源码和示例已存入 `.refs/WinUI/`。

### Status & info
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| InfoBar（可关提示条） | 🚧 | `components/status-info/info_bar.slint`，支持自适应排布、操作按钮插槽、自定义内容和可取消的关闭事件 |
| InfoBadge（角标） | 🚧 | `components/status-info/info_badge.slint`，支持点、计数、字形与图像，并可附在导航条目上 |
| ProgressBar | 🚧 | `components/status-info/progress_bar.slint`，支持确定和不确定模式、暂停与错误 |
| ProgressRing（环形） | 🚧 | `components/status-info/progress_ring.slint`，支持确定与不确定模式、范围、配色和停用 |
| ToolTip | 🚧 | `components/status-info/tool_tip.slint`，支持延迟、文本与自定义内容、位置偏移和避让 |

#### InfoBar

运行 `cargo run -p lumatune --example info_bar` 可试用四种严重度、普通按钮、超链接、纯内容提示条及关闭取消。组件默认收起，由宿主设置 `open`。标题、消息和操作位根据测量宽度排成一行或多行，操作组件在切换时保留实例。默认内容插槽单独布局，只有内容时放在首行。超链接操作通过 `action-is-hyperlink` 使用 WinUI 的缩进。

Windows 浅色 100% 和 125% 已实测四种配色、常规与堆叠布局、长消息、两种操作、RTL、图标与关闭按钮显隐、悬停提示、键鼠激活、可取消关闭和重新打开。原生 Gallery 已通过 UI Automation 对照普通按钮、长消息堆叠和超链接位置。Slint 的 Windows UI Automation 提供 Region、分级 LiveSetting 和关闭按钮 Invoke。WinUI 提供 StatusBar 角色与独立的打开、关闭通知，实际读屏播报、原生通知语义和 Acrylic 提示底材仍待验收或实现。[官方模板](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/InfoBar/InfoBar.xaml) · [布局算法](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/InfoBar/InfoBarPanel.cpp)

#### InfoBadge

运行 `cargo run -p lumatune --example info_badge` 可试用强调色和五种状态色、4×4 点状徽标、16px 高的计数、状态字形和自定义图像。非负 `value` 优先显示数字，负值使用图标或点状状态。自定义图像可保留原色。导航条目通过 `badge` 提供可见性、内容和状态描述，顶部布局会按徽标宽度重新计算。

Windows 浅色已验证六组样式、零值与最大 32 位整数、计数优先级、显隐切换、展开和紧凑窗格、顶部导航、RTL，以及徽标与导航点击的共存。独立徽标可向辅助技术提供非焦点文本，嵌入导航时则作为该条目的状态描述。Slint 会把小于 -1 的值按无数字状态渲染，WinUI 对这类输入抛出异常。Windows 状态字形依赖 Segoe Fluent Icons，读屏播报尚待验收。官方模板及示例已存入 `.refs/WinUI/`。

#### ProgressBar

运行 `cargo run -p lumatune --example progress_bar` 可试用确定进度、不确定进度、暂停、错误、RTL 和内边距。`value` 接收原始进度，`effective-value` 提供范围内的结果。范围放宽后会恢复原始请求值；最小值超过最大值时，最大值随之上调。错误优先于暂停。

Windows 浅色已与 WinUI Gallery 对照，并实测 1px 轨道、六种状态、两段扫描、宽窄尺寸、内边距、RTL 和范围恢复。UI Automation 在确定模式报告只读范围值，在不确定模式报告忙碌、暂停或错误状态。按偶数尺寸约定，默认指示条由 3px 取整到 4px。确定进度的条宽立即更新，原生重新定位和恢复运行的过渡仍有差异。原生模板设置和读屏播报待补齐。模板、行为源码及 Gallery 示例已存入 `.refs/WinUI/`。

#### ProgressRing

运行 `cargo run -p lumatune --example progress_ring` 可切换确定与不确定模式、透明与浅灰轨道、动画和活动状态，并试用 16/32/60px 三种尺寸。范围边界交叉时，刚设置的边界同步收拢另一端，进度值限制在区间内。两端相等时显示空进度。

Windows 浅色已实测尺寸、轨道、升降动画、活动状态和自定义范围。确定样式已与 WinUI Gallery 实例对照，圆弧从 12 点方向顺时针增长。UI Automation 在活动时提供进度条角色，并在确定模式提供只读范围值。Slint 使用矢量圆弧并按项目约定取偶数尺寸。WinUI 的自定义 `IAnimatedVisualSource` 和读屏播报仍待补齐。模板、动画源码、自动化实现及 Gallery 示例已保存在 `.refs/WinUI/`。

#### ToolTip

运行 `cargo run -p lumatune --example tool_tip` 可试用 Gallery 的普通提示、负垂直偏移、图像避让、长文本和 RTL。挂载在宿主窗口内容的最后一层，目标、放置矩形和指针使用该层的同一坐标系。自动提示响应悬停和键盘焦点，支持延迟显示、超时和主动关闭。示例读取 Windows 悬停时长与通知显示时长。InfoBar、NavigationView、PagerControl 和 PipsPager 的悬停提示共用样式，仍由 Slint 的附加提示负责触发。

浅色 100% 和 125% 下已实测文字换行及 320px 宽度上限、悬停延迟、超时后防止立即重开、键盘、Escape、禁用目标、手动开关、偏移、左右避让与指针穿透。原生 Gallery 的键盘提示位于目标上方，间距为 12px。原生 `ToolTip` 无障碍角色、触摸长按、多目标快速重显示、跨窗口弹层和真实 Acrylic 材质仍待补齐。[官方主题资源](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/CommonStyles/ToolTip_themeresources.xaml) · [显示行为](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/dxaml/xcp/dxaml/lib/ToolTipService_Partial.cpp)

### Scrolling
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| ScrollView / ScrollViewer | 🚧 | `components/scrolling/scroll_view.slint`、`scroll_bar.slint`，支持覆盖滚动条、锚点缩放、滚动链、对齐与程序滚动 |
| PipsPager | 🚧 | `components/scrolling/pips_pager.slint`，支持有限和无限分页、横竖方向、循环与按需显示前后按钮 |
| AnnotatedScrollBar | 🚧 | `components/scrolling/annotated_scroll_bar.slint`，支持标记、详细提示、拖动、连发箭头及滚动联动 |
| PagerControl | 🚧 | `components/scrolling/pager_control.slint`，支持自动、下拉、数字输入和页码按钮模式 |
| SemanticZoom | 🚧 | `components/scrolling/semantic_zoom.slint`，支持详情与分组概览、定位映射和焦点恢复 |

运行 `cargo run -p lumatune --example pips_pager` 可切换横向、纵向、RTL、有限或无限页数，以及始终显示、悬停或键盘聚焦时显示、折叠三种箭头策略。有限页数可循环翻页。当前页会在可见范围内居中，靠近首尾时对齐边缘。方向键移动焦点，Enter、空格与鼠标点击提交选择。超出范围的有限页索引会归入可用范围。

Windows 浅色已实测分页点尺寸、首尾隐藏、选中点移动、鼠标、键盘、循环、无限分页、减少页数时的回退、空页和禁用。UI Automation 已读到分页点的名称和选中态。Slint 使用列表和可选择条目表达分页状态，WinUI 使用 Menu 自动化角色与选择提供程序。WinUI 样式模板替换、读屏播报和手柄仍待验收。[WinUI 接口定义](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/PipsPager/PipsPager.idl) · [自动化实现](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/PipsPager/PipsPagerAutomationPeer.cpp)

#### ScrollView / ScrollViewer

运行 `cargo run -p lumatune --example scroll_view` 查看原图滚动、程序滚动、自定义时长、等速滚动和嵌套视口。宿主提供未缩放的内容尺寸和正文插槽。偏移使用物理方向的视口像素，RTL 把垂直滚动条移到左侧。嵌套控件通过 `scroll-chained` 将边界外的滚轮请求交给父视口。ScrollViewer 保留单独的默认轴策略与 `change-view` 入口。

对照了 Gallery 的 400×266px 视口、覆盖式 12px 滚动条、紧凑滑块、8px 展开滑块和两秒收起时间。浅色 100% 和 125% 下检查滚轮、Shift 横滚、键盘、Ctrl 滚轮与程序锚点缩放、拖动、隐藏、RTL、等距吸附、自定义时长、连续滚动和嵌套滚动链。FemtoVG 和软件渲染均检查了放大后滚到图像末端的像素。Windows UI Automation 验证了滚动范围、步长和程序设值。ItemsView 与 ListView 共用该视口。

Slint 软件渲染器忽略通用缩放变换。该后端的照片示例关闭 `transform-content`，用 `zoom-factor` 计算正文尺寸。原生 Composition 惯性与回弹、系统偏好动态订阅、滚动锚点候选、区域及非等距吸附点、外部控制器和原生双轴 UI Automation ScrollPattern 仍待对齐。现在通过两个范围控件暴露滚动值。Pinch 和触摸平移已接入 Slint 手势，触屏实机与手柄仍待验证。模板、行为、Gallery 示例和照片保存在 `.refs/WinUI/`。[ScrollView 模板](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/ScrollView/ScrollView.xaml) · [Gallery ScrollView 示例](https://github.com/microsoft/WinUI-Gallery/blob/6de91feccadfbb0a796b71caedead5a2284f730e/WinUIGallery/Samples/ScrollView/ScrollViewPage.xaml)

#### AnnotatedScrollBar

运行 `cargo run -p lumatune --example annotated_scroll_bar` 可试用 Gallery 色块集合、轨道高度切换和滚动同步。宿主注册 `AnnotatedScrollBarBehavior`，按偏移升序传入标签，把当前滚动位置绑定到 `value`，并在 `scroll-requested` 中应用新位置。标签位置按整个内容范围计算，重叠时按原生顺序优先保留靠下的标签，并尽量保留首项。详细提示可以请求到视口底部所对应的内容偏移。宿主可在 `scrolling` 中取消单次请求。

已与 Windows 浅色 Gallery 对照 30px 指示条、颜色、箭头与标签位置及浮动提示的锚点。实测按下跳转、拖动、触摸释放、取消、滚轮和键盘通过正文反向同步、箭头连发与 UI Automation Invoke、缩短轨道和 RTL。指示条由 3px 取整为 4px。箭头默认步进为首次使用时视口高度的八分之一。示例用回调连接 Slint 滚动视口，原生组合线程滚动、连续惯性、自定义标签模板、跨窗口详情提示与读屏仍待补齐。模板、源码和 Gallery 示例已保存在 `.refs/WinUI/`。

#### PagerControl

运行 `cargo run -p lumatune --example pager_control` 可试用有限和无限页数、显示模式、边界按钮及 RTL。页码索引从零起算，界面从一显示。自动模式在 10 页切换为数字输入，负页数表示无限页，收起总页数和末页按钮。边界隐藏保留箭头的位置，始终隐藏会回收位置。宿主注册 `PagerControlBehavior` 和下拉、数字框所需的行为，将焦点遍历发往控件所在的窗口。四个导航按钮先更新选中页，再发各自的回调。

已与 Windows 浅色 Gallery 对照 40px 导航按钮、64px 数字框、页码按钮和选中线。实测 9／10 页切换、数值提交与取整、下拉选择、首尾禁用与保留间距、方向键移动焦点、按钮面板的三段省略号、无限页、RTL 和 UI Automation Invoke。原生 5px 间距按项目约定取 6px。无限下拉沿用原生的 100 个选项，自动模式可用数字输入继续翻页。极大页码的数字框受 Slint 单精度限制。原生按钮样式对象、菜单自动化角色、手柄和读屏仍待补齐。模板、源码和 Gallery 示例已存入 `.refs/WinUI/`。

#### SemanticZoom

运行 `cargo run -p lumatune --example semantic_zoom` 可试用分组详情、概览、锁定、RTL 和可选缩小按钮。示例从 `.refs/WinUI/` 读取 Gallery 控件目录，缓存缺失时仍可使用内置分组。宿主通过 `zoomed-in-view` 和 `zoomed-out-view` 插槽提供视图，以稳定条目和分组键实现 `location-at` 与 `map-destination`，并在 `focus-destination` 中滚动和聚焦目标。自带滚动容器的插槽可先把 Ctrl+滚轮交给 `handle-wheel`。

已与 Windows 浅色 Gallery 对照分组标题、双列详情和概览列表，实测鼠标切换、Ctrl+-、Ctrl++、Ctrl+滚轮、双指缩放、定位和焦点恢复。Slint 在双指手势结束后提交切换。原生实时缩放过渡、触控板捏合、平台返回键自动接入和 UI Automation Toggle 模式仍有差异。

### Menus & toolbars
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| MenuBar / MenuBarItem | 🚧 | `components/menus-toolbars/menu_bar.slint`，含标题、悬停切换、级联菜单、访问键和范围快捷键 |
| CommandBar / AppBarButton / AppBarToggleButton / AppBarSeparator | 🚧 | `components/menus-toolbars/command_bar.slint` 与 `app_bar_button.slint`，支持主次命令、动态溢出、自定义内容和附属菜单。 |
| CommandBarFlyout | 🚧 | `components/menus-toolbars/command_bar_flyout.slint`，复用 AppBar 元素并支持标准、瞬态和指针移远关闭模式。 |
| SwipeControl | 🚧 | `components/menus-toolbars/swipe_control.slint`，支持双向 Reveal 或 Execute、触点匹配及操作回调 |

#### MenuBar

运行 `cargo run -p lumatune --example menu_bar` 可试用 File、Edit、Help 和级联命令。菜单栏和弹层使用同一个带稳定 ID 的平面菜单模型，顶层条目为子菜单标题。将菜单栏放在宿主内容上层并传入窗口视口，使用 `MenuBarAccelerators` 包裹需要访问键及快捷键的区域。

已对照原生 WinUI Gallery 的 40px 菜单栏、32px 标题、标题间距、下方贴边及 40px 命令步距。浅色 100% 和 125% 下实测鼠标按下、悬停切换、外点关闭、方向键、Escape 后恢复焦点、Alt 访问键、输入框内快捷键、禁用和 RTL 级联。Windows UI Automation 已验证展开、顶层切换与活动标题失效时收起。Slint 以 Group 与 Button 暴露菜单栏和标题。原生 MenuBar/MenuItem 角色、`ExpandCollapse.Collapse` 转发、Alt 键提示、触屏及跨窗口弹层仍待补齐。[官方模板](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/MenuBar/MenuBarItem.xaml) · [键鼠行为](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/MenuBar/MenuBarItem.cpp)

#### CommandBar

运行 `cargo run -p lumatune --example command_bar` 可试用命令栏。附加 `-- --narrow --bottom` 可查看窄窗口和向上展开，`-- --narrow --ordered` 可查看指定顺序的成组迁移，`-- --app-bar-buttons` 可单独试用三种 AppBar 元素。已验证浅色下的紧凑与最小状态、向上和向下展开、方向键、Esc、外部点击关闭、快捷键、溢出区内滚动和附属菜单。实际调整窗口大小时，切换命令迁入和移出溢出区后保留选中状态。Windows UI Automation 已实测展开、调用和切换。[Microsoft CommandBar 规范](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/command-bar) · [WinUI 溢出与键盘源码](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/CommandBar_Partial.cpp) · [命令栏模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommonStyles/CommandBar_themeresources.xaml) · [按钮模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommonStyles/AppBarButton_themeresources.xaml)

展开状态使用 Acrylic 的纯色回退，溢出区在窗口内绘制。长标签换行、`AppBarElementContainer` 和分裂命令、跨窗口弹层、触屏和手柄、RTL 目检及读屏播报仍待补齐或验收。

运行 `cargo run -p lumatune --example command_bar_flyout` 可试用两种打开方式。附加 `-- --edge` 可试用向上展开，`-- --pointer-dismiss` 可试用指针移远关闭，`-- --always-expanded`、`-- --primary-only` 和 `-- --secondary-only` 可试用其余命令布局。浅色已验证标准模式起始焦点、瞬态保留输入焦点、省略号双向切换、Tab 与方向键、外部点击、指针距弹层超过 80px 后关闭、上下文键和附属菜单。执行次命令可关闭浮层并恢复到调用方指定的焦点。Windows UI Automation 已实测展开、切换和调用。[Microsoft CommandBarFlyout 规范](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/command-bar-flyout) · [WinUI 键盘逻辑](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommandBarFlyout/CommandBarFlyoutCommandBar.cpp) · [WinUI 呈现模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommandBarFlyout/CommandBarFlyout_themeresources.xaml)

当前浮层随宿主窗口绘制，使用 Acrylic 的纯色回退。宿主通过 `restore-focus` 指定恢复元素，用 `CommandBarFlyoutKeyScope` 处理瞬态下的 Esc。原生菜单自动化角色、跨窗口定位、自定义命令容器、触屏和手柄仍待补齐或验收。

#### SwipeControl

运行 `cargo run -p lumatune --example swipe_control` 可试用横向和纵向滑动、执行后保持打开、渐变和位图图标。附加 `-- --scroll-list` 可在十二行列表里复验滑动操作与触屏滚动。宿主通过内容插槽和各侧的 `SwipeItems` 提供内容与操作，同一控件选择横轴或纵轴。每个 Execute 侧接收一项操作。`close()` 可关闭保持打开的 Execute。宿主可在点到控件之外或同组其他项开始滑动时调用 `dismiss()`，以保留该状态。

已对照 Windows 浅色 Gallery 的 500×68px 示例、68px 操作项、图标和标签的排列，以及 Execute 跨过阈值前后的配色。实测长短滑动、反向关闭、松手单次执行、第二指加入后的取消、同组关闭、列表滚动和删除。示例额外支持 Ctrl+方向键、Tab 和 Esc，展开的操作可通过 Windows UI Automation Invoke 调用。内容插槽里的交互子控件优先接收指针输入。WinUI 组合线程的速度投影、触控板手势、窗口级自动外点关闭及读屏播报仍有差异。模板、行为源码、Gallery 示例和位图已保存到 `.refs/WinUI/`。

### Text
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| TextBlock | ✅ | `Text` |
| TextBox / LineEdit | 🚧 | `components/text/text_box.slint`，复用 Slint 文本编辑，提供 WinUI 字段样式、单行和多行行为 |
| PasswordBox | 🚧 | `components/text/password_box.slint`，支持掩码字符、Peek／Hidden／Visible 和密码命令菜单 |
| TextCommandBarFlyout | 🚧 | `components/text/text_command_bar_flyout.slint`，根据文本控件、选区、剪贴板和编辑历史显示命令。 |
| NumberBox | 🚧 | `components/text/number_box.slint`，支持隐藏、内联与紧凑步进，以及表达式、空值、范围和验证模式。 |
| AutoSuggestBox（建议下拉） | 🚧 | `components/text/auto_suggest_box.slint`，支持异步筛选、键鼠选择、文本预览及查询提交。 |
| RichTextBlock / RichTextBlockOverflow | 🚧 | 段落与行内格式、文本高亮、跨容器续排、键鼠选区和链接 |
| RichEditBox | 🚧 | 原生 Rich Edit 文档、浮动文本命令栏、拼写更正、UnicodeMath 与 MathML、RTF、格式、撤销、占位符与查找高亮 |

运行 `cargo run -p lumatune --example text_command_bar_flyout` 可试用可编辑文本、只读文本和密码输入框。右键、菜单键和 Shift+F10 打开上下文菜单，“Show selection tools” 演示选区工具栏。浅色已验证选区在菜单夺取焦点后仍可见，Esc 后恢复编辑焦点，次命令执行后收起，以及撤销、重做和系统剪贴板上的 Unicode 剪切、复制和粘贴。只读示例仅显示复制和全选，密码示例仅显示可用的粘贴和全选。Windows UI Automation 已验证调用全选和展开次命令。[Microsoft 文档及截图](https://learn.microsoft.com/windows/apps/develop/ui/controls/command-bar-flyout) · [WinUI 命令规则](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/CommandBarFlyout/TextCommandBarFlyout.cpp)

编辑器负责提供选区、可粘贴状态和撤销重做能力。示例管理逐步编辑历史。实机触控自动打开及选区定位、富文本编辑器接入、拼写检查和读屏播报仍待补齐或验收。

运行 `cargo run -p lumatune --example number_box` 可试用 NumberBox，附加 `-- --dark` 切换至深色。已验证 Enter 与失焦提交、Esc 恢复、上下键和 PageUp/PageDown、焦点内滚轮、内联与紧凑步进、长按连续步进、环绕、两种验证模式、范围限制、长输入光标和程序设值。Windows UI Automation 已实测 Spinner 类型、范围属性和设值。[Microsoft NumberBox 规范](https://learn.microsoft.com/windows/apps/develop/ui/controls/number-box) · [WinUI 控件实现](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/NumberBox/NumberBox.cpp) · [WinUI 呈现模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/NumberBox/NumberBox.xaml)

当前公开数值使用 Slint 的 32 位浮点，空值由 `has-value` 表示。十进制分隔符跟随 Slint 区域设置，支持自定义解析和格式化回调。仍需补齐 WinUI `Double` 的精度和地区格式、编辑器右键菜单、紧凑步进面板跨窗口绘制和读屏播报。

运行 `cargo run -p lumatune --example auto_suggest_box` 可试用 AutoSuggestBox，附加 `-- --above` 可将编辑器放在窗口底部，触发建议列表向上展开。已验证键盘选择、Esc 和 Tab 恢复原始文本、Enter 和鼠标提交、搜索图标提交、禁用建议文本预览、清空、禁用和异步筛选。Windows UI Automation 已实测编辑框类型与设值、展开状态和列表项类型。共享输入框样式后复验了 NumberBox 的键鼠交互和长文本光标滚动。[Microsoft AutoSuggestBox 规范](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/auto-suggest-box) · [WinUI 控件实现](https://github.com/microsoft/microsoft-ui-xaml/blob/main/dxaml/xcp/dxaml/lib/AutoSuggestBox_Partial.cpp) · [WinUI 呈现模板](https://github.com/microsoft/microsoft-ui-xaml/blob/main/controls/dev/AutoSuggestBox/AutoSuggestBox_themeresources.xaml)

建议弹层当前在父容器内绘制，使用 Acrylic 的纯色回退。跨窗口与父容器裁剪、建议项模板、IME 候选窗避让、手柄和读屏播报仍待补齐或验证。

#### TextBox

运行 `cargo run -p lumatune --example text_box` 可试用普通输入、标题与占位符、只读格式和自适应高度的多行文本。宿主注册 `TextBoxBehavior` 后可启用 UTF-16 最大输入长度。输入视口跟随光标滚动，窄字段、只读字段和多行字段隐藏清除按钮。示例通过现有 TextCommandBarFlyout 提供上下文命令。编辑器处理撤销与重做快捷键，菜单里的可用状态由宿主提供。

已与 Windows 浅色 Gallery 对照 64×32px 的空白字段、输入后宽度变化、聚焦线、清除按钮和占位区域。实测单行 Enter、多行换行与滚动、宽度限制、只读和禁用、RTL、菜单选区恢复及 Windows UI Automation 编辑角色与设值。TextBox 复用 Slint 的输入法提示。WinUI 的系统拼写检查、手写视图和自定义标题模板仍有差异。模板、核心实现和 Gallery 示例已保存在 `.refs/WinUI/`。

#### PasswordBox

运行 `cargo run -p lumatune --example password_box` 可试用标题、占位符、掩码字符、显隐模式、禁用和窄宽状态。宿主注册 `PasswordBoxBehavior` 后可使用自定义掩码与 UTF-16 最大输入长度。Peek 只在当前聚焦期间从空值录入文本后出现，按住按钮或 Alt+F8 显示，松开与失焦恢复掩码。已有密码或较窄字段不出现 Peek 按钮。Windows 的禁止显示密码策略用于 Peek 模式。

已与 Windows 浅色 Gallery 对照 300×32px 字段、聚焦线、掩码与按住显示的位置。实测按住和释放、Alt+F8、重新聚焦、限制输入、自定义掩码及 Visible。Windows UI Automation 在显示期间仍报告密码角色，Value 和 Text 提供程序未读出内容。已验证复制和剪切快捷键不写入 Windows 剪贴板，模拟剪贴板回归覆盖粘贴、截断和取消。文本命令栏示例已接入 PasswordBox，菜单沿用密码字段命令集。WinUI 的系统 PIN 输入范围、原生读屏提示及内存加密仍有差异。模板、核心实现和 Gallery 示例已保存到 `.refs/WinUI/`。

#### RichTextBlock

运行 `cargo run -p lumatune --example rich_text_block` 可试用简单段落、粗体和斜体、链接、行内图像、连续分栏及文本高亮。宿主注册 `RichTextBehavior` 并提供段落与行内格式。DirectWrite 按各容器的宽度换行，完整的行依序放入容器。绘制、选区和指针定位共用该排版结果，位置和文本高亮范围使用 UTF-16 偏移。最后一个容器仍有溢出时可读取剩余文本。选区跨容器时按文档顺序复制，链接激活和剪贴板交给宿主。示例接入共享文本命令栏的复制和全选。

已对照本地 WinUI Gallery 的浅色正文、格式化文字、链接及三栏两端对齐。100% 和 125% 缩放下实测链接点击、绿色拖选、Ctrl+A、Shift+方向键、右键及 Shift+F10 的命令菜单、跨列选区、最后一列的溢出状态、RTL 及黄、红、蓝三种范围高亮。Windows 排版回归测试覆盖不同宽度的续排、1.25 倍位图及行内图像度量。Slint 将链接暴露为可调用按钮。WinUI 原生 Hyperlink、UI Automation TextPattern、手柄和触摸选择、自定义行内控件及独立控件间的溢出连接仍有差异。官方控件源码和示例已保存在 `.refs/WinUI/`。

#### RichEditBox

运行 `cargo run -p lumatune --example rich_edit_box` 可试用简单编辑框和带工具栏的编辑器，附加 `-- --sample-text` 可预填包含中文和 Emoji 的文本。Slint 复用文本字段边框，Windows Rich Edit 子窗口负责输入、选择、滚动和文档。宿主可访问 UTF-16 选区、切换粗体、斜体和下划线、设置前景色和使用系统剪贴板。示例通过文件对话框读写 RTF，查找可从当前选区继续并回绕。打开时先在独立编辑器中解析 RTF，保存时在目标目录写完临时文件再替换。

空文档在原生排版区域显示占位符，聚焦时仍可见，输入后隐藏。原生子窗口通过 UI Automation HelpText 提供占位提示。查找框聚焦时显示全部匹配，更新查询或移开焦点时清除旧标记。标记按原生文本范围位置绘制，实测匹配跨换行时两行都显示标记。选区、文档格式、撤销记录和 RTF 保持原样。当前用系统高亮色半透明叠加，Gallery 使用实色背景并交换文字色。[Microsoft 文档中的文本范围几何](https://learn.microsoft.com/en-us/windows/win32/api/tom/nf-tom-itextrange2-getrect)

鼠标拖选后，在选区上方显示共享文本命令栏。主栏提供粗体、斜体和下划线，更多菜单按文档状态提供剪切、复制、粘贴、撤销、重做和全选。移出主栏 80px 后自动收起。右键和键盘上下文菜单会打开带有可用次级命令的浮动栏，按 Esc 将焦点交回编辑器。弹层覆盖之处临时裁切原生子窗口，其余正文、选区和输入继续由 Rich Edit 处理。Windows 回归测试覆盖裁切、复原及选区保留。[WinUI 命令分组](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/controls/dev/CommandBarFlyout/TextCommandBarFlyout.cpp) · [WinUI 弹层的输入设备行为](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/dxaml/xcp/dxaml/lib/FlyoutBase_partial.cpp)

运行 `cargo run -p lumatune --example rich_edit_box -- --math` 可打开 Gallery 对应的两个数学示例。Windows Rich Edit 使用 Cambria Math 原生编辑和呈现 UnicodeMath，模式切换时清空正文和撤销历史。“Set sample formula” 导入 MathML 并替换原公式。导入先在隐藏子窗口解析，失败时保留现有正文和选区。MathML 代码区随第二个编辑区更新，跨行时为空。Windows 11 23H2 及更早版本的数学模式依赖系统的受限访问功能。[RichEditBox 数学模式](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/rich-edit-box#use-a-rich-edit-box-for-math-equations) · [Windows 数学模式 API](https://learn.microsoft.com/en-us/uwp/api/windows.ui.text.richeditmathmode) · [Rich Edit MathML 格式标志](https://devblogs.microsoft.com/math-in-office/setting-and-getting-text-in-various-formats/)

Windows 11 24H2 浅色主题下，100% 和 125% 缩放均实测了分式、上标、导入替换、实时导出及长公式换行后清空预览。也通过原生 UI Automation 核对了两个数学编辑区的名称和 TextPattern。回归测试覆盖无效 MathML 保留正文与选区、退出数学模式恢复普通文本输入，以及预览内数学符号的缩进。

普通编辑区使用 Windows 拼写服务。可用的更正放在「Spelling」子菜单中，打开错词的上下文菜单时自动展开。更正前核对 UTF-16 范围内的原文，使已过期的建议失效。更正可以撤销。运行 `cargo run -p lumatune --example rich_edit_box -- --spelling` 可试用错词标记和更正，示例优先选用已安装的 en-US 词典，其次使用系统拼音词典。显式选词典时，示例按原生排版范围绘制红色波浪线。[WinUI 拼写菜单实现](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/dxaml/xcp/core/native/text/Controls/TextBoxBase.cpp#L5517-L5646) · [Windows 拼写检查选项](https://learn.microsoft.com/en-us/windows/win32/controls/em-setlangoptions)

浅色 100% 和 125% 下用本机拼音词典实测 `beijng` 的红线、鼠标和键盘打开子菜单、替换为 `Beijing`、还原焦点及撤销。回归测试还核对了表情符号后的 UTF-16 范围、过期建议，以及命令栏与子菜单同时覆盖原生编辑区时的裁切。当前系统只安装了拼音词典，en-US 拼写结果待在装有该词典的系统上复验。

错词可添加到对应语言的 Windows 用户词典，或在当前编辑器会话里忽略该词。重复词可删除或只忽略当前位置。浅色 100% 和 125% 下实测忽略错词会去掉所有同词标记，忽略重复词只作用于当前一处，删除重复词可以撤销。回归测试核对了忽略和添加词条前后的正文、选区和撤销状态，并覆盖混合格式和语言、忽略范围随前文移动、整篇替换清除单次忽略。词典写入通过隔离回调验证词条和语言，本次没有实际改动系统词典。[WinUI 校对操作](https://github.com/microsoft/microsoft-ui-xaml/blob/905a4f6cb8e810d086a1382101cf19c4ce1f61a9/dxaml/xcp/core/native/text/Controls/TextContextMenu.cpp#L258-L313) · [Windows 用户词典写入](https://learn.microsoft.com/en-us/windows/win32/api/spellcheck/nf-spellcheck-ispellchecker-add)

浅色示例按本地 Gallery 的 32px 简单字段和 200px 自定义编辑区制作。100% 和 125% 缩放下实测 Unicode 输入、占位符、选择格式、浮动命令栏、撤销重做、只读、查找高亮、Tab 从自定义编辑区进入 Slint 查找框和文件对话框的 RTF 往返。两个原生子窗口直接通过 UI Automation 提供名称和 TextPattern。原生子窗口使用 Document 角色，Gallery 使用 Edit 角色。Gallery 已核对自动化尺寸和模板，实机截图为空白，像素对照仍待复验。触摸和笔的选区入口、停止自动更正及从 Slint 无障碍树发现原生编辑区仍待处理。已裁切的弹层轮廓之外由原生窗口绘制，弹层阴影不能叠在该部分正文上。源码和示例已保存在 `.refs/WinUI/`。

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
2. 🚧 InfoBar（`status-info/info_bar.slint`，已验证操作位、布局切换和关闭事件，待验收读屏与原生通知）
3. 🚧 Expander（`layout/expander.slint`，已验证双向展开、标题插槽及内容禁用，待补原生无障碍折叠与自动禁用继承）
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
