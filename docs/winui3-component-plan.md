# WinUI 3 组件复刻目标

复刻范围：**WinUI 3 本体（Microsoft.UI.Xaml）自带控件**。不含 CommunityToolkit 扩展（`DataGrid`、`SettingsCard`、`SettingsExpander` 等属 Toolkit，非本体）。

## 约定

- 放在 `apps/lumatune/ui/components/`，与 `nav_rail.slint`、`breadcrumb_bar.slint`、`content_dialog.slint` 同一套风格：`export component`、属性/回调注入、Slint `Palette` 语义色。
- **长度一律偶数**，计算结果取最近偶数（奇点为界，取较大偶数，如 1→2、3→4、2.5→2）。
- 图标用 SVG `@image-url(...)` + `colorize` 染色（见 `apps/lumatune/ui/icons/`）。
- 每个组件：编译无警告 + 截图目检 + 至少一处可运行示例验证后再提交。
- 状态标记：⬜ 待做 / 🚧 进行中 / ✅ 完成 / ⏭️ 跳过（不做）。

## 现状图例

✅ 已有可直接用（Slint `std-widgets`）· 🟡 部分可用需增强 · 🔨 需复刻 · ⚠️ 不建议/难等价

Slint 1.18.1 Fluent 主题已导出：`Button, CheckBox, ComboBox, GroupBox, LineEdit, RadioGroup, ListView, StandardListView, ProgressIndicator, Slider, SpinBox, Spinner, TabWidget, Switch, TextEdit, TimePickerPopup, DatePickerPopup, ScrollView, VerticalBox/HorizontalBox/GridBox, Palette`。

## 可行性矩阵

### 布局 / 结构
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| Grid / StackPanel / RelativePanel / Canvas / Viewbox | ✅ | `GridBox/HorizontalBox/VerticalBox` + 矩形布局足够 |
| SplitView | 🔨 | 侧边抽屉 + 遮罩 |
| NavigationView | 🔨 | 导航壳；项目已自研 `NavRail` 覆盖部分 |
| TabView（可增删/关闭标签） | 🟡 | `TabWidget` 仅静态标签，无关闭/新建 |
| Expander（折叠区块） | 🔨 | 高价值 |
| Pivot | 🔨 | 可用 `TabWidget` 近似 |
| ItemsRepeater | 🟡 | `for`/`ListView` 覆盖多数 |

### 内容 / 导航
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| Frame / Page（路由） | 🔨 | 项目现用 `if page==` 手搓 |
| NavigationView | 🔨 | 见上 |

### 文本
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| TextBlock | ✅ | `Text` |
| RichTextBlock / RichEditBox | ⚠️ | 富文本支持弱 |
| HyperlinkButton | 🔨 | 无链接按钮 |
| ContentLink | 🔨 | |

### 按钮 / 选择
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| Button / RepeatButton | ✅ | |
| CheckBox | ✅ | |
| RadioButton / RadioButtons（分组） | 🟡 | `RadioGroup` 有，`RadioButtons` 便捷分组需封装 |
| ToggleSwitch | 🟡 | `Switch` 有，样式不完全同 WinUI |
| ToggleButton / DropDownButton / SplitButton / ToggleSplitButton | 🔨 | 皆无 |
| ComboBox / ComboBoxItem | ✅ | |
| Slider | ✅ | |
| RatingControl（星级） | 🔨 | |
| ColorPicker | ⚠️ | 复杂，暂不做 |

### 输入
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| TextBox / PasswordBox | 🟡 | `LineEdit` 有；password 需掩码封装 |
| NumberBox（带 ± 步进） | 🔨 | 高价值；`SpinBox` 可包装 |
| AutoSuggestBox（建议下拉） | 🔨 | 高价值 |
| CalendarDatePicker / DatePicker / TimePicker | 🟡 | 底层 `DatePickerPopup/TimePickerPopup` 有弹层，缺完整 picker |

### 菜单 / 浮层
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| MenuBar / MenuBarItem | 🟡 | 公开有 `MenuBar` |
| MenuFlyout / Flyout | 🔨 | 任意锚点浮层需封装 |
| CommandBar / AppBarButton / AppBarToggleButton / AppBarSeparator | 🔨 | 高价值（含 overflow …） |
| TeachingTip（引导气泡） | 🔨 | |
| ToolTip | 🟡 | 基础可用，样式弱 |
| ContentDialog（模态框） | ✅ | 复刻完成：`components/content_dialog.slint` |

### 信息 / 状态
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| InfoBar（可关提示条，含严重度） | 🔨 | 高价值 |
| InfoBadge（角标） | 🔨 | |
| ProgressBar | 🟡 | `ProgressIndicator` |
| ProgressRing（环形进度） | 🔨 | `Spinner` 是转圈非环形进度 |
| PipsPager（分页点） | 🔨 | |
| AnimatedVisualPlayer（Lottie） | ⚠️ | 依赖 Lottie 运行时 |

### 数据
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| ListView | ✅ | |
| GridView | 🟡 | `ListView`/`for`+wrap 可拼 |
| TreeView | 🔨 | 无 |
| FlipView（轮播） | 🔨 | |

### 媒体 / 其他
| 控件 | 现状 | 备注 |
| --- | --- | --- |
| Image | ✅ | |
| SymbolIcon / FontIcon / PathIcon / IconSourceElement | ✅ | 已用 SVG `@image-url` 体系等价 |
| PersonPicture（头像） | 🔨 | 低难度 |
| WebView2 / MediaPlayerElement | ⚠️ | 宿主能力，难原生等价 |

## 复刻优先级（实用 + Slint 缺）

1. ⬜ ContentDialog —— ✅ 已完成（`components/content_dialog.slint`）
2. ⬜ InfoBar
3. ⬜ Expander
4. ⬜ CommandBar
5. ⬜ NumberBox / AutoSuggestBox
6. ⬜ NavigationView（或在现有 NavRail 上补齐）
7. ⬜ TabView（在 TabWidget 上补关闭/新建）
8. ⬜ InfoBadge / PipsPager / PersonPicture（小而独立）
9. ⬜ Flyout / MenuFlyout / TeachingTip
10. ⬜ TreeView / RatingControl / ProgressRing / HyperlinkButton

## 每个组件的验收标准

- 编译通过且无警告；`cargo test` 通过。
- 软件渲染截图目检：布局、配色、交互态（hover/press/disabled/暗色）符合 WinUI 3 视觉。
- 至少一处可运行示例（临时插入预览、验证后移除）证明实际渲染正确。
- 遵循本文件顶部的样式约定。
