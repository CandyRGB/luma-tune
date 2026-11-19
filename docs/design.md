# LumaTune 设计口径

这份文档收敛设计访谈的全部结论。凡是与它冲突的实现，都以这里为准；要改，先改这里对应的 ADR。

## 一句话

Windows 上的画面调校工具，用显示层查表改善游戏暗部可见度与观感，不注入任何进程，因此对反作弊完全无感。

## 边界：明确不做的事

| 不做 | 原因 |
| --- | --- |
| 进程注入、Present 钩子、DXGI 代理 | ADR-0001。反作弊风险不可接受，且单人开发周期以季度计 |
| 饱和度、振动、锐化、黑白、色盲模拟 | ADR-0004。需要跨通道运算或厂商驱动接口 |
| HDR 支持 | 系统在 HDR 下整体忽略查表。检测到就禁用并提示，不静默无响应 |
| `.cube` 导入 | 把 3D LUT 拟合成三条一维曲线只能得到「大概像」，沟通成本高于功能价值 |
| 低级键盘钩子 | 会被反作弊与杀软按可疑行为处理，为热键弄脏产品叙事不值得 |
| 曝光 (Exposure) | 编码空间下与「Gamma + 亮度」等价，多一个滑杆只增加困惑 |
| 账号、云同步、社区预设、OBS WebSocket | 无服务端，v1 不碰网络 |

## 核心模型

滤镜 = 一条固定顺序的**算子链**（ADR-0003）。用户调的是算子参数，不是曲线本身。每个算子是一元标量函数，对三条通道各算一遍，最终得到 256 项查表。

```
输入黑位/白位 → 输出黑位/白位 → 亮度 → 对比度 → Gamma → 三通道增益 → 曲线 → 钳位
```

所有运算在**编码空间**进行（ADR-0002）——不做 sRGB 解码，因为这个查表下游根本不存在色彩管理。

参数定义：

| 参数 | 数学 | 范围 | 中性 |
| --- | --- | --- | --- |
| 输入黑位 | `(x − lo) / (hi − lo)` 的 lo | 0…0.5 | 0 |
| 输入白位 | 同上，hi | 0.5…1 | 1 |
| 亮度 | `x + b` | −0.12…0.12 | 0 |
| 对比度 | `(x − 0.5)·c + 0.5` | 0.2…3.0 | 1 |
| 暗部 | `x + a·x·(1 − smoothstep(0, 0.7, x))` | −0.3…0.3 | 0 |
| 高光 | `x − a·smoothstep(0.4, 1, x)` | −0.3…0.3 | 0 |
| Gamma | `x^(1/g)` | 0.2…5.0 | 1 |
| 色温 | 1000K…10000K 的三通道增益 | 见左 | 6500K |
| 色调 | 绿—品红轴增益 | −1…1 | 0 |
| 通道增益 | R/G/B 各自乘性 | 0.5…1.5 | 1 |
| 输出黑位 | `f + (1 − f)·x` 的 f | 0…0.15 | 0 |
| 输出白位 | `wh·x` 的 wh | 0.7…1 | 1 |
| 曲线 | 最多 8 个控制点 | 单调 | 恒等 |

暗部与高光的权重都锚在端点上：暗部在纯黑处权重为零，所以它永远只提亮暗部而不把黑变成灰；高光只压上半段，纯黑不动。高光会把白点按 `a` 的量压下来，这正是它和输出白位的分工——前者是「让高光收敛」，后者是「决定白最终停在哪」。

曲线必须用**保单调插值**（PCHIP / Fritsch–Carlson），不能用普通三次样条——后者会在控制点之间过冲，产生肉眼可见的色带和反相。

亮度的上限被压到 ±0.12，因为加性偏移会直接吃掉一端：`b = −0.3` 意味着 30% 的暗部被压成死黑，没有任何实用性。这个范围由 `FilterParams::evaluate` 强制执行，预设文件里写了更大的值也会被夹住，不会渲染出碎裂的曲线。

输出量化到 `WORD`（0…65535）。Windows 会校验查表，非法值返回 `STATUS_GRAPHICS_INVALID_GAMMA_RAMP (0xC01E0347)`。

## 失效检测

`SetDeviceGammaRamp` 在部分适配器上**静默失败**：不报错，屏幕不动。这是这类工具最隐蔽的坑，必须主动检测。

- 写入前用 `GetDeviceGammaRamp` 读回当前表，作为恢复基线与比对依据。
- 写入后用 `GetDeviceGammaRamp` 读回，与期望值比对（允许驱动量化误差）。
- 读回结果与写入前**完全相同**，即判定该适配器不支持，UI 明确提示而不是假装成功。
- HDR 检测走 `QueryDisplayConfig` 的 `DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO`，`advancedColorEnabled` 为真即禁用滤镜。

## 状态生命周期

改的是系统状态，不是自己的窗口。程序死了，状态还在。所以：

- 写入非恒等表**之前**先落一个脏标记文件，成功写回恒等表**之后**删除。
- 启动时若发现脏标记，无条件先写一次恒等表，再进入正常流程。
- 生效中的曲线是一个**栈**（ADR-0005）：按应用触发时压入，退出时弹出并恢复上一层，栈空才回恒等表。用户手动调的那条不会被游戏触发清掉。
- 全局热键 `Ctrl+Alt+Shift+R` 强制清栈并写回恒等表，不依赖主窗口可见。

## 显示器

每台显示器独立曲线。身份用 EDID 派生的稳定标识（厂商 + 产品码 + 序列号），**不用 `\\.\DISPLAY1` 这类序号**——换线或改分辨率会让序号重排，预设全部错位。无序列号的屏退化到「分辨率 + 位置」哈希。

预设可以不绑定显示器（表示全部适用）。

## 预设

`%APPDATA%\LumaTune\presets\*.json`，一个预设一个文件。损坏隔离，可直接分享，可进版本控制。

```json
{
  "schemaVersion": 1,
  "name": "暗部增强",
  "display": null,
  "bindings": ["cs2.exe"],
  "operators": {
    "levels": { "inBlack": 0.0, "inWhite": 1.0, "outBlack": 0.0, "outWhite": 1.0 },
    "brightness": 0.0,
    "contrast": 1.0,
    "gamma": 1.0,
    "gains": { "r": 1.0, "g": 1.0, "b": 1.0 },
    "temperature": 6500,
    "tint": 0.0,
    "curve": [[0.0, 0.0], [0.25, 0.22], [0.5, 0.5], [0.75, 0.78], [1.0, 1.0]]
  }
}
```

只存参数，不存渲染结果。`.cube` 只支持导出，用于把参数变成可分享资产。

已实现于 `crates/lumatune-core/src/preset.rs`（数据结构与校验）和 `crates/lumatune-win/src/presets.rs`（读写）。`display` 与 `bindings` 两个字段已经存在于 schema 中，但当前版本还不会填它们——多显示器绑定与按应用触发是下一步。

## 进程触发

`CreateToolhelp32Snapshot` 每 1.5 秒轮询一次，按 exe 文件名匹配绑定列表。不用 WMI（延迟不稳、可能需提权），不用焦点钩子（启动器与中间层进程没有窗口）。

多个绑定进程同时运行时，后进入的压在栈顶。

## 界面

Slint 1.18，控件风格用内置的 **fluent**，不显式指定样式——编译器的默认值就是它（`typeloader.rs` 里 `unwrap_or_else(|| "fluent".into())`），并且它会跟随系统深浅色。

既然跟随系统，配色就**不能硬编码**。界面上所有颜色都走 `Palette` 语义色：`background`（内容层，浅色下 #FAFAFA）/ `alternate-background`（导航边栏，浅色下 #F0F0F0）/ `foreground` / `accent-background`（可点击的文字与强调）/ `border`（控件描边），次要文字用 `foreground.with-alpha(...)` 降透明度。硬编码任何 `#rrggbb` 都会在另一半配色下失效。

窗口是**三层灰白**，与 WinUI `NavigationView` 的图层一致：

| 层 | 令牌 | 浅色值 | 对应 WinUI |
| --- | --- | --- | --- |
| 导航边栏 / 标题带 | `background` + 3% 前景覆盖 | #F2F2F2（242） | 窗格底（acrylic 扁平化后的灰） |
| 内容区 | `background` | #FAFAFA（250） | `LayerFillColorDefault` |
| 卡片 | `background` + 1px `Theme.card-stroke` | #FAFAFA + 10% 前景描边 | `CardBackgroundFillColorDefault` + `CardStrokeColorDefault` |

导航底与标题带之所以不是直接用 `alternate-background`（#F0F0F0 = 240），是因为对着官方示例取色发现它的窗格是 **243–244**，我们的要暗 3–4 级。现在用「`background` 打底 + 3% 前景覆盖」压到 **242**，与官方基本重合；深色下则变成 35（内容 28），层次比原来的 44 更接近 WinUI 那种若隐若现的分层。

卡片与内容区同色是 WinUI 的做法：卡片不靠填色区分，靠**细描边 + 8px 圆角 + 2px 偏移的 5% 阴影**。所以卡片描边不能用 `Palette.border`——那是 45% 黑的控件描边，用在卡片上过重；`Theme.card-stroke`（10%）与 `Theme.divider`（8%）是应用内定义的两档细线。

**唯一的硬编码颜色例外**：设置页里「跟随系统 / 浅色 / 深色」三张缩略图的配色写死为 `#fafafa` / `#1c1c1c` 等值。缩略图是**对某种模式的描绘**，深色缩略图在浅色主题下也必须是深色，否则它就无法表达自己代表什么。

## 导航边栏组件与动效

主窗口使用 `NavigationView`。`menu-items` 承载「预设库」和「应用」，`settings` 将「设置」固定在底部。应用以 `NavPage` 保存当前页面，并用稳定 ID 与 `selected-id` 同步。`item-invoked` 切换页面，`back-requested` 在预设编辑页返回列表。

默认窗口使用 200px 宽的展开窗格，中等宽度收至 48px，窄窗口使用覆盖窗格。窗格切换直接采用目标布局。下面的性能数据来自早期宽度动画实验。

### 实测开销

Slint 自带 `SLINT_DEBUG_PERFORMANCE=refresh_lazy,console` 会打印帧率、后端与渲染层数，用它直接量。

后端确认是 **FemtoVG + OpenGL**（不是软件渲染）。窗口 1040×640 逻辑 / 1300×800 物理。

| 指标 | 数值 |
| --- | --- |
| 静止 | 0–2 fps（只在必要时重绘，空闲不烧 CPU） |
| 展开 / 收缩动画中（debug） | 15–24 fps |
| 展开 / 收缩动画中（release） | 24–35 fps |
| 单次 180ms 过渡的 CPU（release） | **约 190 ms** |

测法：当时内置了一个基准钩子（每 250 ms 切换一次 `compact`，跑 3 秒），用进程累计 CPU 时间对比空闲跑（两者都含约 0.5 s 启动开销）。release 下空闲 0.59 s，动画 2.88 s，差值 2.28 s ÷ 12 次 = 190 ms/次。该钩子已随截图钩子一并删除，表中数字是删除前测得的历史数据。

对成本的归因试了两项，都不显著：

- **`clip: true` 的离屏层**：去掉后每帧层数从 2 降到 0，帧率没变（debug 仍是 16–23），说明瓶颈不在合成层。
- **卡片阴影**：去掉 5 处 `drop-shadow`，动画期间 CPU 降约 11%（3.58 s vs 3.90 s，debug），不值得为它牺牲层次。

**结论**：宽度动画去掉了，不是因为 CPU 不够（单次 180 ms 过渡只花约 0.19 秒单核时间，任务管理器里根本看不出来），而是**帧时间不够**——这套界面的单帧成本 25–40 ms 决定了任何牵动布局的动画都跑不到 60 fps，看起来就是卡。省 CPU 和看起来流畅是两件事，这里卡在后者。

颜色与形变类过渡保留：它们只重绘局部，帧时间远小于布局动画，观感上没有掉帧感。帧率上限是 UI 渲染量（滑杆、卡片阴影、渐变、圆角）决定的，要提帧得减渲染量，例如把滑杆改成静态绘制、卡片阴影换成纯描边。

## 面包屑（BreadcrumbBar）

`NavigationView` 的 `page-header` 插槽显示当前页面的路径。预设列表显示「预设库」，编辑页显示「预设库 › 预设名」或「预设库 › 新建预设」，应用与设置页各显示一项。

`BreadcrumbBar` 使用 `items: [Crumb]` 接收路径。宽度不足时，省略号收起最左侧的祖先，浮出菜单以由近到远的顺序列出这些祖先。`select(int)` 回传祖先在原路径中的索引。末项显示当前位置。

预设库使用独立的列表和参数编辑页。点击「预设库」面包屑、导航返回按钮或在编辑时再次选择「预设库」会调用同一返回逻辑，清除 UI 选中项并通知 Rust 清除当前预设的索引。

## 主题模式

设置页「外观 → 应用模式」提供三选一，改动立即生效并持久化。

难点在于 `Palette` 是 std-widgets 的全局量，Rust 侧拿不到（`ui.global::<Palette>()` 编译不过），所以切换只能写在 Slint 里——

```slint
function apply-theme() {
    Palette.color-scheme = theme-mode == 1 ? ColorScheme.light : (theme-mode == 2 ? ColorScheme.dark : ColorScheme.unknown);
}
```

两个要点：

- `Palette.color-scheme` 在 fluent 样式里是 `<=> FluentPalette.color-scheme` 的双向绑定。运行时赋值会**打断**这条链，之后系统主题变化不再自动传进来。
- 所以「跟随系统」不能只是不改，而必须显式写回 `ColorScheme.unknown`。`FluentPalette.dark-color-scheme` 对 `unknown` 的处理是回落到 `SlintInternal.color-scheme`，也就是平台上报的系统值——链路由此恢复。实测「深色 → 跟随系统」在同一个进程内能正确回到浅色。

持久化：`%APPDATA%\LumaTune\settings.json`，内容 `{ "theme": "system" | "light" | "dark" }`。文件缺失或损坏时回落 `system`，不阻塞启动（`lumatune-win::settings`，4 个单测覆盖缺失 / 往返 / 损坏 / 索引映射）。

托盘常驻 + 主窗口（左预设列表，右参数面板），不做浮层——独占全屏下任何窗口都不可见。

界面采用 Windows 11 设置页的卡片结构：组标题（光照 / 色彩）在卡片**外面**，卡片是 `border-radius: 8px` 的圆角块（`Palette.background`，1px `Theme.card-stroke` 描边）。参数行高 40px，行间用缩进 16px 的 1px 分隔线，首行不画线。

导航窗格的行高度与收起宽度由 `NavigationView` 统一控制，应用在编辑预设时启用返回键。组件的交互验证记录见 [WinUI 3 组件复刻计划](winui3-component-plan.md#navigationview)。

尺寸遵循 WinUI 规范：控件圆角 4px、卡片圆角 8px、标准控件高 32px、导航项高 36px。

图标用 **Segoe Fluent Icons** 的字体字形，`icons.slint` 把语义枚举翻译成码位，`Icon` 组件继承 `Text`：

```slint
import { Icon, Icons } from "icons.slint";

Icon { icon: Icons.save; size: 16px; tint: Palette.foreground; }
```

| 成员 | 说明 |
| --- | --- |
| `enum Icons` | 语义名（`Icons.save` 等），界面里不出现码位 |
| `component Icon` | 继承 `Text`：`text` 由 `icon` 三元选出码位，`font-family: "Segoe Fluent Icons"`、`font-size: size`、`color: tint`；外框 `size + 4`、双向居中 |

| 枚举 | 码位 | 枚举 | 码位 |
| --- | --- | --- | --- |
| `Icons.library` | `e8f1` | `Icons.save` | `e74e` |
| `Icons.apps` | `e8a9` | `Icons.trash` | `e74d` |
| `Icons.settings` | `e713` | `Icons.compare` | `e8ab` |
| `Icons.add` | `e710` | `Icons.reset` | `e72c` |
| `Icons.cancel` | `e711` | `Icons.back` | `e72b` |
| `Icons.menu` | `e700` | `Icons.chevron` | `e76c` |

加图标：在 [Segoe Fluent Icons 码位表](https://learn.microsoft.com/windows/apps/design/style/segoe-fluent-icons-font) 里查到码位 → 枚举尾部加成员 → `text` 三元链加一支。

**为什么最终选了字体而不是 SVG**：SVG 方案要维护 1721 个文件（3.1 MB）、每个图标一句 `@image-url`、着色靠 `colorize`、留白靠外层盒子躲开 `image-fit: contain` 的缩放陷阱；字体方案一行一个码位，尺寸就是 `font-size`，颜色就是 `color`，没有缩放、没有解码。渲染上字形由 DirectWrite 按设备像素光栅化并做灰度校正，观感比 resvg 的线性 alpha 混合更轻、更贴 Win11 本体。

**代价**：这套字形依赖系统字体（Win11 自带；Win10 只有 Segoe MDL2 Assets，码位不完全兼容），换到 macOS / Linux 会退化成方块。若将来真要跨平台，正确解法是内嵌 `FluentSystemIcons-Regular.ttf`（MIT，2.9 MB）再改 `font-family`——但 Slint 桌面端没有稳定的自定义字体注册 API，`import "xxx.ttf"` 只在 MCU 目标下生效，桌面要开 `unstable-fontique-011` 特性调 `shared_collection().register_fonts()`。原先的 1721 个 SVG（提交 `fd0d3c8`）已从工作区删除，需要时从历史里取。

模块只依赖 `std-widgets` 的 `Palette`（`tint` 默认值），不依赖应用组件，可直接复制到别的 Slint 工程。

行的排布仿设置页：标签靠左定宽 108px，滑块定宽 240px 与数值一起右对齐，中间留白。滑块不撑满整行——14 个满幅滑杆会变成一堵蓝色长条。

参数区高度固定，内容超出时滚动，不靠加大窗口解决。滚动容器用 `Flickable` 而不是 `ScrollView`，并且必须设 `mouse-drag-pan-enabled: false`：该属性单独控制「鼠标按住拖动视口」，关掉之后鼠标拖拽才会落到滑块上；`interactive` 管的是触摸平移，保持默认。右侧自绘一条 6px 滚动指示条，位置由 `content-y` 与 `content-height` 推算。

Spacing 只用 8 和 16 两档，不出现零散数值。

**Slint 没有原生托盘支持**，需要 `tray-icon` + `muda` 补上。这是本项目 Slint 侧最大的未知量，应尽早验证。

参数拖动即写入（节流到 30ms），另提供 A/B 对比键：按住临时切恒等表，松开恢复。人眼对绝对亮度不敏感、对差异敏感，没有对照物就调不准。

## 验收标准

在一个真实游戏里跑通完整闭环：

1. 启动游戏，进程被识别，预设自动套用。
2. 切出去调参，画面实时变化，A/B 对比可用。
3. 退出游戏，曲线回到触发前那条，而不是恒等表。
4. 强杀本程序，下次启动能自愈，屏幕不残留偏色。

这四条之外的单测和手点都不算数——产品的全部价值都在这四个时刻。

## 实测记录

2026-11-13，Lenovo 笔记本，NVIDIA GeForce GTX 1650，1920x1080，SDR。探针见 `crates/gamma-probe`。

| 项 | 结果 |
| --- | --- |
| 查表可用 | 是 |
| 写入后读回 | 768/768 逐项一致，最大误差 0，**读回可作校验依据** |
| 精度 | 16 位，驱动未量化到 8 位 |
| `SetDeviceGammaRamp` | 中位 3.5 ms，最大 4.8 ms |
| `GetDeviceGammaRamp` | 中位 1.7 ms，最大 3.5 ms |
| 恢复原表 | 逐项一致 |

延迟 3.5 ms 是这份数据里唯一的坏消息，它直接否掉两件事：**参数变更不能做逐帧过渡动画**（60 fps 下会吃掉两成单核），**实时拖动必须节流且窗口不能低于 10 ms**。一次写入加一次读回校验共 5.2 ms，仍可放在 30 ms 节流窗口里。

仍然待验：

- 曲线肉眼是否可见（`gamma-probe visual`）。读回一致只能证明驱动接受了数据，不能证明显示器真的照着显示。
- Slint + `tray-icon` 的事件循环能否共存。

已验：**无边框全屏下查表不被覆盖**。120 秒连续写入、每 500 ms 校验，前台在 `QUNS_BUSY`（Windows 11 全屏优化下的 flip 模型，现代游戏的常态）与普通窗口之间切换，重置次数为 0。真正的 D3D 独占全屏（`QUNS_RUNNING_D3D_FULL_SCREEN`）没有覆盖到，但该形态正被微软逐步淘汰，暂不阻塞开发。

真正需要担心的不是系统重置，而是游戏自己写一整条查表把它顶掉——D3D9 那一代引擎在启动和切场景时会调 `SetGammaRamp`。这条只有在具体游戏里才会暴露，`watch` 的校验能捕捉到，留待实测。

## 已知实现约束

进程必须声明 per-monitor DPI 感知（`SetProcessDpiAwarenessContext`），否则 `MONITORINFO` 返回的是被 125% 缩放虚拟化后的坐标——探针第一版就踩到了，1920x1080 被报成 1536x864。

虚拟显示适配器（远程串流、投屏软件创建的）会让 `SetDeviceGammaRamp` 表现出与真实显卡完全不同的延迟和精度，检测到适配器名字含 virtual / indirect / basic 时应在 UI 里提示结果不可靠。


## 技术选型

`slint` · `windows`（Gdi / Dxgi / Threading / WindowsAndMessaging）· `serde` + `serde_json` · `tray-icon` + `muda` · `directories` · `tracing`
