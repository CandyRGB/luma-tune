# LumaTune

<p align="center">
  <b>简体中文</b> · <a href="README.md">English</a>
</p>

> 一款面向游戏场景的 Windows 画面调校工具。LumaTune 提供显示调校和无边框窗口画面后处理，无需向游戏进程注入代码。

LumaTune 是一个基于 [Slint](https://slint.dev/) 与 [`windows`](https://crates.io/crates/windows) crate 构建的小型桌面应用。光照和色彩调校通过 256 项的 `Gamma Ramp` 应用于显示器。

- **画面后处理** —— 绑定应用进入前台时应用预设中非零的 HDR 色调、散光、清晰度、饱和度和自然饱和度，游戏继续接收鼠标和键盘输入。
- **预设库** —— 用卡片浏览和收藏预设，自定义主图。初始预设支持恢复默认参数。
- **多显示器** —— 在主页逐屏应用预设或恢复原始显示。启动时恢复上一次手动应用的配置。
- **按应用触发预设** —— 将预设绑定到可执行文件，应用切到前台时自动套用。
- **异常恢复** —— 下次启动时，按显示器恢复异常退出前保存的原始曲线，再加载桌面预设。
- **实时 A/B 对比** —— 在预设编辑页选择显示器或窗口，拖动分割线比较同一帧画面应用草稿预设前后的效果。

## 画面后处理

后处理需要 Windows 10 2004 或更新版本、SDR 显示器和无边框窗口模式。在「预设库」编辑预设，调节「画面后处理」里的参数并保存，再在「应用绑定」选择该预设。HDR 色调调节局部明暗，散光为亮部添加柔和光晕。清晰度范围为 −1 到 1，主要柔化或增强中间调的局部明暗反差。饱和度范围为 −1 到 1，−1 为灰度，0 为原饱和度，正值让颜色更鲜艳。自然饱和度范围为 0 到 1，优先增强偏灰的颜色。各项数值均为 0 时会直接显示原游戏窗口。后处理通过 Windows Graphics Capture 读取绑定窗口，再用 Direct3D 11 绘制到鼠标穿透的覆盖层。切出游戏时覆盖层自动关闭。

## 显示调校原理

滤镜是一条固定顺序的**算子链**。用户调的是算子参数，而不是曲线本身。每个算子是一元标量函数，对三条通道各算一遍，最终得到 256 项查表：

```
输入黑位/白位 -> 输出黑位/白位 -> 亮度 -> 对比度 -> Gamma -> 三通道增益 -> 曲线 -> 钳位
```

所有运算在**编码空间**进行（不做 sRGB 解码），因为下游查表根本不存在色彩管理。末端的曲线采用保单调插值（PCHIP / Fritsch–Carlson），避免控制点之间出现过冲、色带与反相。

| 参数 | 范围 | 中性 |
| --- | --- | --- |
| 输入黑位 / 白位 | 0…0.5 / 0.5…1 | 0 / 1 |
| 亮度 | −0.12…0.12 | 0 |
| 对比度 | 0.2…3.0 | 1 |
| 暗部 / 高光 | −0.3…0.3 | 0 |
| Gamma | 0.2…5.0 | 1 |
| 色温 / 色调 | 1000K…10000K / −1…1 | 6500K / 0 |
| 通道增益 | 0.5…1.5 | 1 |
| 输出黑位 / 白位 | 0…0.15 / 0.7…1 | 0 / 1 |
| 曲线 | 最多 8 个控制点 | 恒等 |

> [!NOTE]
> 饱和度通过画面后处理调节。显示查表按通道分别计算。检测到 HDR 时会提示滤镜的限制。

## 明确不做的事

拒绝进程注入、Present 钩子与 DXGI 代理——反作弊风险不可接受。低级键盘钩子、`.cube` LUT 导入、把曝光当作独立参数，以及任何账号、云同步、社区预设等需要服务端的功能，v1 都不涉及。

## 仓库结构

```
apps/lumatune          Slint 桌面应用（UI 与运行时装配）
crates/lumatune-core   与平台无关的滤镜模型：参数、算子、曲线、预设
crates/lumatune-win    Windows 适配层：显示器调校、窗口捕获与后处理、设置和存储
crates/gamma-probe     独立诊断 CLI，用于验证查表支持与延迟
docs/                  设计说明、参考资料与 ADR
```

## 构建与运行

> [!IMPORTANT]
> LumaTune 仅支持 Windows。构建需要稳定版 Rust、MSVC C++ 构建工具和包含 C++/WinRT 头文件的 Windows SDK。

```sh
# 构建整个工作区
cargo build --release

# 运行桌面应用
cargo run -p lumatune --release

# 在本机验证查表支持与读回精度
cargo run -p gamma-probe --release -- report
```

`gamma-probe` 还提供 `visual`（屏幕曲线检查）、`watch`（连续写入/校验循环）、`focus`（前台进程检测）、`restore`（写回恒等表）等子命令。

桌面应用的导航组件来自已发布的 [`slint-winui` crate](https://crates.io/crates/slint-winui)。

关闭主窗口后，LumaTune 留在 Windows 通知区域继续调校显示器。左击托盘图标可重新打开窗口。右击图标并选择「退出 LumaTune」会结束程序并恢复显示设置。

### Windows 安装包

构建 x64 程序时静态链接 C/C++ 运行库，再使用 NSIS 3 打包。

```powershell
cargo --config 'target.x86_64-pc-windows-msvc.rustflags=["-C","target-feature=+crt-static"]' build --locked --release -p lumatune --target x86_64-pc-windows-msvc
makensis /INPUTCHARSET UTF8 installer/LumaTune.nsi
```

安装包生成在 `target/LumaTune-<版本>-Setup-x64.exe`。首次安装默认使用 `C:\Program Files\LumaTune`，可另选路径。升级沿用已登记的安装位置。安装程序比较本机与随包可执行文件的 Windows 文件版本，本机已有相同或更新版本时会跳过安装。更新或卸载前，请从托盘退出 LumaTune。

## 测试

```sh
cargo test
```

核心模型（参数、曲线插值、预设）与 Windows 设置层带有单元测试。端到端闭环的正确性靠手动验收：启动游戏确认预设自动套用，切出实时调参，退出确认曲线回到触发前那条，再强杀进程确认重启后自愈。

## 数据与文件

设置与预设存放在 `%APPDATA%\LumaTune\` 下。`display-recovery.json` 在调校前记录每块显示器的原始曲线，恢复成功后清除对应条目。

## 文档

- 设计口径与验收标准：`docs/design.md`
- 架构决策记录：`docs/adr/`
- 术语表：`GLOSSARY.md`
