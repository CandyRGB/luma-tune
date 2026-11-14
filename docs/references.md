# 设计参考

`.refs/` 存放外部仓库的只读副本，只作为视觉与规范的对照材料，**不参与构建，也不提交**（已在 `.gitignore` 中忽略）。里面的代码版权归各自作者，不要复制进本项目。

## WinUIonWeb

`https://github.com/Furry-Xiyi/WinUIonWeb` 是对 Microsoft WinUI 的 Web 复刻，用 Vue 实现。技术栈与本项目无关，**代码不可复用**；有价值的是它把 WinUI 的资源字典原样搬成了 CSS 变量，可以当作配色、圆角、控件度量的对照表。

稀疏检出，只取了两个目录：

| 路径 | 用途 |
| --- | --- |
| `.refs/WinUIonWeb/WinUIonWeb/src/styles/theme.css` | 主题资源。用 WinUI 官方 brush 名（`CardBackgroundFillColorDefaultBrush`、`TextFillColorPrimaryBrush` 等）映射到简化变量，逐控件列出浅色/深色两套取值 |
| `.refs/WinUIonWeb/WinUIonWeb/src/styles/buttonResources.css` | 按钮的完整状态矩阵（normal / pointerover / pressed / disabled），可以对照 Slint fluent 样式的默认表现 |
| `.refs/WinUIonWeb/WinUIonWeb/src/components/` | 各控件的模板与样式，看具体控件怎么组织视觉层次 |

重新拉取（如果 `.refs/` 被清掉）：

```sh
git clone --depth 1 --filter=blob:none --sparse \
  https://github.com/Furry-Xiyi/WinUIonWeb.git .refs/WinUIonWeb
cd .refs/WinUIonWeb
git sparse-checkout set WinUIonWeb/src/styles WinUIonWeb/src/components --skip-checks
```

## 已采用的规范值

从该仓库统计得出，已应用到 `apps/lumatune/ui/app.slint`：

- 控件圆角 **4px**（WinUI `ControlCornerRadius`）
- 卡片圆角 **8px**（`OverlayCornerRadius`）
- 标准控件高 **32px**
- 导航项高 **36px**

配色不照抄——本项目统一走 Slint 的 `Palette` 语义色，它本身也是从 WinUI 调色板派生的。只有在需要确认「某个场景在 WinUI 里应该用什么语义色」时才来这份 theme.css 里查。
