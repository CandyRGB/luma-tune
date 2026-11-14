mod display;
mod foreground;
mod ramp;

use std::io::{self, Write};
use std::time::{Duration, Instant};

use display::Display;
use ramp::Ramp;
use windows::Win32::Graphics::Gdi::{DeleteDC, HDC};

const WARMUP: usize = 8;
const SAMPLES: usize = 200;

fn main() {
    unsafe {
        let _ = windows::Win32::System::Console::SetConsoleOutputCP(65001);
        let _ = windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
            windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        );
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("report") => report(args.iter().any(|a| a == "--all")),
        Some("visual") => visual(number_arg(&args, 1)),
        Some("watch") => watch(number_arg(&args, 1), number_arg(&args, 2)),
        Some("focus") => focus(),
        Some("restore") => restore_all(),
        Some("help") | Some("-h") => help(),
        _ => menu(),
    }
}

fn number_arg(args: &[String], at: usize) -> Option<usize> {
    args.get(at).and_then(|s| s.parse().ok())
}

fn width(s: &str) -> usize {
    s.chars()
        .map(|c| if (c as u32) > 0x2e80 { 2 } else { 1 })
        .sum()
}

fn pad(s: &str, n: usize) -> String {
    format!("{}{}", s, " ".repeat(n.saturating_sub(width(s))))
}

fn row(label: &str, value: &str) {
    println!("  {}{}", pad(label, 20), value);
}

fn banner(title: &str) {
    println!();
    println!("{}", title);
}

fn help() {
    println!("gamma-probe [命令]");
    println!();
    println!("  report [--all]            写回一致性、精度、延迟（默认只测主显示器）");
    println!("  visual [序号]             写入肉眼可见的曲线，回车后恢复");
    println!("  watch [序号] [秒数]       持续写入并检测是否被系统重置（默认 60 秒）");
    println!("  focus                     查看前台窗口是全屏还是窗口化");
    println!("  restore                   把所有显示器还原为备份或恒等表");
    println!();
    println!("不带参数进入菜单。");
}

fn menu() {
    println!("LumaTune 显示层探针");
    println!();
    println!("  1  环境报告（主显示器）");
    println!("  2  环境报告（全部显示器）");
    println!("  3  可见曲线测试");
    println!("  4  游戏内持续写入观察");
    println!("  5  前台窗口与全屏状态");
    println!("  6  还原所有显示器");
    println!("  0  退出");
    println!();
    print!("选择：");
    let _ = io::stdout().flush();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return;
    }
    match line.trim() {
        "1" => report(false),
        "2" => report(true),
        "3" => visual(None),
        "4" => watch(None, Some(60)),
        "5" => focus(),
        "6" => restore_all(),
        _ => {}
    }
}

fn focus() {
    let displays = display::enumerate();
    let primary = displays
        .iter()
        .find(|d| d.primary)
        .or_else(|| displays.first());
    let Some(s) = foreground::capture() else {
        println!("没有前台窗口");
        return;
    };
    banner("前台窗口");
    row("标题", if s.title.is_empty() { "-" } else { &s.title });
    row("类名", if s.class.is_empty() { "-" } else { &s.class });
    row("进程", &format!("{}  (pid {})", s.process, s.pid));
    row(
        "窗口矩形",
        &format!("{}x{} @ ({}, {})", s.width, s.height, s.left, s.top),
    );
    let mut style = String::new();
    if s.popup {
        style.push_str("POPUP ");
    }
    if s.caption {
        style.push_str("CAPTION ");
    }
    if s.thick_frame {
        style.push_str("THICKFRAME ");
    }
    if s.layered {
        style.push_str("LAYERED ");
    }
    row("样式", if style.is_empty() { "-" } else { style.trim() });
    row(
        "系统全屏状态",
        &format!(
            "{} ({})",
            foreground::state_label(s.fullscreen_state),
            s.fullscreen_state
        ),
    );
    if let Some(d) = primary {
        row("屏幕", &format!("{}  {}x{}", d.device, d.width, d.height));
        banner("判定");
        println!("  {}", foreground::verdict(&s, d.width, d.height));
    }
}

fn report(all: bool) {
    let displays = display::enumerate();
    if displays.is_empty() {
        println!("没有找到显示器");
        return;
    }
    println!("LumaTune 显示层探针");
    print_displays(&displays);
    let targets: Vec<&Display> = if all {
        displays.iter().collect()
    } else {
        displays.iter().filter(|d| d.primary).collect()
    };
    let mut verdicts = Vec::new();
    for d in targets {
        verdicts.push(probe(d));
    }
    banner("结论");
    for v in verdicts {
        println!("  {}", v);
    }
}

fn print_displays(displays: &[Display]) {
    banner("显示器");
    println!(
        "  {}  {}  {}  {}  {}  {}",
        pad("#", 3),
        pad("设备名", 18),
        pad("分辨率", 12),
        pad("主屏", 5),
        pad("色彩空间", 10),
        "位深"
    );
    for (i, d) in displays.iter().enumerate() {
        let space = if d.hdr() { "非 SDR" } else { "SDR" };
        println!(
            "  {}  {}  {}  {}  {}  {}",
            pad(&(i + 1).to_string(), 3),
            pad(&d.device, 18),
            pad(&format!("{}x{}", d.width, d.height), 12),
            pad(if d.primary { "是" } else { "" }, 5),
            pad(space, 10),
            d.bits_per_color
                .map(|b| b.to_string())
                .unwrap_or_else(|| "-".into())
        );
        let virtual_like = {
            let a = d.adapter.to_lowercase();
            a.contains("virtual") || a.contains("indirect") || a.contains("basic")
        };
        println!(
            "     {}  {}{}",
            pad("适配器", 8),
            if d.adapter.is_empty() {
                "未知".to_string()
            } else {
                d.adapter.clone()
            },
            if virtual_like {
                "   （虚拟适配器，结果不代表真实显卡）"
            } else {
                ""
            }
        );
    }
}

struct Probe {
    device: String,
    dc_ok: bool,
    read_ok: bool,
    exact: usize,
    delta: u16,
    precise16: bool,
    set_us: f64,
    hdr: bool,
}

fn probe(d: &Display) -> String {
    banner(&format!("探针  {}", d.device));
    let mut p = Probe {
        device: d.device.clone(),
        dc_ok: false,
        read_ok: false,
        exact: 0,
        delta: 65535,
        precise16: false,
        set_us: 0.0,
        hdr: d.hdr(),
    };

    let hdc = display::dc_for(&d.device);
    if hdc.0.is_null() {
        row("设备上下文", "取 DC 失败");
        return verdict(&p);
    }
    p.dc_ok = true;

    let Some(original) = ramp::read(hdc) else {
        row("读取当前表", "失败");
        unsafe {
            let _ = DeleteDC(hdc);
        }
        return verdict(&p);
    };
    p.read_ok = true;

    let backup = ramp::save_backup(&d.device, &original);
    row(
        "原表",
        if ramp::is_identity(&original) {
            "恒等"
        } else {
            "非恒等，已存备份"
        },
    );
    if let Some(path) = &backup {
        row("备份路径", &path.display().to_string());
    }
    if d.hdr() {
        row("HDR", "已开启，查表会被系统忽略");
    }

    let pattern = ramp::probe_pattern();
    let wrote = ramp::write(hdc, &pattern);
    row("写入", if wrote { "调用成功" } else { "调用失败" });

    if let Some(rb) = ramp::read(hdc) {
        p.exact = ramp::exact_matches(&pattern, &rb);
        p.delta = ramp::max_delta(&pattern, &rb);
        p.precise16 = !ramp::eight_bit_only(&rb);
        row("读回一致", &format!("{}/768", p.exact));
        row("最大误差", &p.delta.to_string());
        row("精度", if p.precise16 { "16 位" } else { "8 位量化" });
    } else {
        row("读回", "失败");
    }

    let (set_med, set_max) = time_set(hdc, &pattern);
    let (get_med, get_max) = time_get(hdc);
    p.set_us = set_med;
    row(
        "SetDeviceGammaRamp",
        &format!("中位 {:.1} µs   最大 {:.1} µs", set_med, set_max),
    );
    row(
        "GetDeviceGammaRamp",
        &format!("中位 {:.1} µs   最大 {:.1} µs", get_med, get_max),
    );

    let wrote_back = ramp::write(hdc, &original);
    let restored = ramp::read(hdc).is_some_and(|rb| rb == original);
    row(
        "恢复",
        if wrote_back && restored {
            "成功，与原表逐项一致"
        } else {
            "失败，请运行 restore"
        },
    );

    unsafe {
        let _ = DeleteDC(hdc);
    }
    verdict(&p)
}

fn verdict(p: &Probe) -> String {
    if !p.dc_ok {
        return format!("{}  取 DC 失败", p.device);
    }
    if !p.read_ok {
        return format!("{}  读表失败", p.device);
    }
    if p.hdr {
        return format!("{}  HDR 已开启，查表不可用", p.device);
    }
    if p.exact == 0 {
        return format!("{}  写入无效，适配器不支持查表", p.device);
    }
    let check = if p.delta == 0 {
        "读回可作校验依据"
    } else {
        "读回有偏差，校验需用容差"
    };
    let precision = if p.precise16 { "16 位精度" } else { "8 位量化" };
    format!(
        "{}  查表可用 · {} · {} · set {:.1} µs",
        p.device, check, precision, p.set_us
    )
}

fn time_set(hdc: HDC, r: &Ramp) -> (f64, f64) {
    for _ in 0..WARMUP {
        ramp::write(hdc, r);
    }
    let mut v = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let t = Instant::now();
        ramp::write(hdc, r);
        v.push(t.elapsed().as_secs_f64() * 1e6);
    }
    stats(v)
}

fn time_get(hdc: HDC) -> (f64, f64) {
    for _ in 0..WARMUP {
        let _ = ramp::read(hdc);
    }
    let mut v = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let t = Instant::now();
        let _ = ramp::read(hdc);
        v.push(t.elapsed().as_secs_f64() * 1e6);
    }
    stats(v)
}

fn stats(mut v: Vec<f64>) -> (f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = v[v.len() / 2];
    (median, *v.last().unwrap_or(&median))
}

fn visual(index: Option<usize>) {
    let displays = display::enumerate();
    let Some(d) = select(&displays, index) else {
        println!("没有可用显示器");
        return;
    };
    let hdc = display::dc_for(&d.device);
    if hdc.0.is_null() {
        println!("取 DC 失败");
        return;
    }
    let Some(original) = ramp::read(hdc) else {
        println!("读表失败");
        return;
    };
    ramp::save_backup(&d.device, &original);
    if d.hdr() {
        println!("注意：该显示器 HDR 已开启，曲线多半不会生效。");
    }
    ramp::write(hdc, &ramp::brighten(2.2));
    println!("已写入 gamma 2.2 测试曲线到 {}。", d.device);
    print!("按回车恢复：");
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
    ramp::write(hdc, &original);
    let ok = ramp::read(hdc).is_some_and(|rb| rb == original);
    println!("恢复{}", if ok { "成功" } else { "失败，请运行 restore" });
    unsafe {
        let _ = DeleteDC(hdc);
    }
}

fn watch(index: Option<usize>, seconds: Option<usize>) {
    let displays = display::enumerate();
    let Some(d) = select(&displays, index) else {
        println!("没有可用显示器");
        return;
    };
    let hdc = display::dc_for(&d.device);
    if hdc.0.is_null() {
        println!("取 DC 失败");
        return;
    }
    let Some(original) = ramp::read(hdc) else {
        println!("读表失败");
        return;
    };
    ramp::save_backup(&d.device, &original);

    let total = Duration::from_secs(seconds.unwrap_or(60) as u64);
    let interval = Duration::from_millis(500);
    let target = ramp::brighten(1.35);
    println!("持续写入 {} 秒，每 500 毫秒校验一次。", total.as_secs());
    println!("切进游戏后若出现「被重置」，说明全屏下曲线会被系统覆盖。");
    println!();

    let started = Instant::now();
    let mut tick = 0u32;
    let mut changes = 0u32;
    let mut last_state = i32::MIN;
    while started.elapsed() < total {
        let state = foreground::capture()
            .map(|s| s.fullscreen_state)
            .unwrap_or(-1);
        if state != last_state {
            last_state = state;
            println!(
                "[{}] 前台 {}",
                stamp(started.elapsed()),
                foreground::state_label(state)
            );
        }
        if tick > 0 {
            if let Some(current) = ramp::read(hdc) {
                if ramp::max_delta(&current, &target) > 512 {
                    changes += 1;
                    println!(
                        "[{}] 被重置  当前表 {}",
                        stamp(started.elapsed()),
                        if ramp::is_identity(&current) {
                            "恒等"
                        } else {
                            "非本次写入的值"
                        }
                    );
                }
            }
        }
        ramp::write(hdc, &target);
        tick += 1;
        std::thread::sleep(interval);
    }

    ramp::write(hdc, &original);
    let ok = ramp::read(hdc).is_some_and(|rb| rb == original);
    println!();
    println!(
        "结束。{} 次校验中 {} 次发现被重置。恢复{}",
        tick,
        changes,
        if ok { "成功" } else { "失败，请运行 restore" }
    );
    unsafe {
        let _ = DeleteDC(hdc);
    }
}

fn restore_all() {
    let displays = display::enumerate();
    if displays.is_empty() {
        println!("没有找到显示器");
        return;
    }
    for d in &displays {
        let hdc = display::dc_for(&d.device);
        if hdc.0.is_null() {
            println!("  {}  取 DC 失败", d.device);
            continue;
        }
        let (label, source) = ramp::load_backup(&d.device)
            .map(|(_, r)| ("备份", r))
            .unwrap_or_else(|| ("恒等表", ramp::identity()));
        let ok =
            ramp::write(hdc, &source) && ramp::read(hdc).is_some_and(|rb| rb == source);
        println!(
            "  {}  {}{}",
            d.device,
            label,
            if ok { "  已还原" } else { "  还原失败" }
        );
        unsafe {
            let _ = DeleteDC(hdc);
        }
    }
}

fn select<'a>(displays: &'a [Display], index: Option<usize>) -> Option<&'a Display> {
    match index {
        Some(i) if i >= 1 && i <= displays.len() => displays.get(i - 1),
        _ => displays
            .iter()
            .find(|d| d.primary)
            .or_else(|| displays.first()),
    }
}

fn stamp(d: Duration) -> String {
    let secs = d.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}
