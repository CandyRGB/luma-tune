# 不引入显卡厂商的色彩接口

参照产品的 Saturation 与 Vibrance 并不走 Gamma Ramp，而是调用 NVIDIA Digital Vibrance 与 AMD Saturation，因此在 Intel 核显与多数笔记本屏上不可用。照做的成本很小，且同样不涉及进程注入，反作弊风险不变。

决定不引入 NVAPI、ADL 等厂商接口。v1 不提供饱和度与振动滑杆。

## Consequences

- 功能面上缺两个玩家会第一时间寻找的滑杆，这是明确的取舍，不是遗漏。
- 项目保持零厂商 DLL 依赖，崩溃面与杀软误报面都更小。
- 若未来要补，饱和度必须作为独立后端接在算子链**之后**，不能混进算子链——它不是一个逐通道算子。

## Considered Options

- **接 NVAPI / ADL**：功能对齐度更高，但引入对闭源厂商 DLL 的 unsafe FFI，且 Intel 平台仍然无解。
- **曲线式伪饱和度**：不需要任何外部依赖，但必然连带改变对比度，与参照产品 1.6.1 之后的承诺冲突。
