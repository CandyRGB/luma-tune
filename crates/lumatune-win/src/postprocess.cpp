#ifndef NOMINMAX
#define NOMINMAX
#endif
#include <windows.h>
#include <d3d11.h>
#include <d3dcompiler.h>
#include <dwmapi.h>
#include <dxgi1_6.h>
#include <roapi.h>
#include <windows.graphics.capture.interop.h>
#include <windows.graphics.directx.direct3d11.interop.h>
#include <winrt/Windows.Foundation.h>
#include <winrt/Windows.Graphics.Capture.h>
#include <winrt/Windows.Graphics.DirectX.Direct3D11.h>

#include <algorithm>
#include <atomic>
#include <chrono>
#include <cstdint>
#include <memory>

namespace {
using winrt::check_bool;
using winrt::check_hresult;
using winrt::com_ptr;
using namespace winrt::Windows::Graphics::Capture;
using winrt::Windows::Graphics::DirectX::DirectXPixelFormat;
using winrt::Windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
using Clock = std::chrono::steady_clock;
using namespace std::chrono_literals;

constexpr wchar_t overlay_class[] = L"LumaTunePostProcessOutput";
constexpr char shader_source[] = R"(
Texture2D<float4> scene : register(t0);
Texture2D<float4> soft_scene : register(t1);
SamplerState bilinear : register(s0);
cbuffer Settings : register(b0) {
    float2 axis;
    float tone;
    float bloom;
    float saturation;
    float clarity;
    float vibrance;
};
struct Fragment { float4 position : SV_POSITION; float2 uv : TEXCOORD0; };
Fragment vertex(uint id : SV_VertexID) {
    Fragment output;
    output.uv = float2((id << 1) & 2, id & 2);
    output.position = float4(output.uv * float2(2, -2) + float2(-1, 1), 0, 1);
    return output;
}
float4 blur(Fragment f) : SV_TARGET {
    const float weights[5] = {0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216};
    float3 color = scene.Sample(bilinear, f.uv).rgb * weights[0];
    [unroll] for (int i = 1; i < 5; ++i) {
        float2 delta = axis * i;
        color += (scene.Sample(bilinear, f.uv + delta).rgb
            + scene.Sample(bilinear, f.uv - delta).rgb) * weights[i];
    }
    return float4(color, 1);
}
float4 compose(Fragment f) : SV_TARGET {
    const float3 weights = float3(0.2126, 0.7152, 0.0722);
    float3 color = scene.Sample(bilinear, f.uv).rgb;
    if (tone != 0 || bloom != 0 || clarity != 0) {
        float3 neighborhood = soft_scene.Sample(bilinear, f.uv).rgb;
        float luminance = dot(neighborhood, weights);
        if (clarity != 0) {
            float pixel_luma = dot(color, weights);
            float detail = pixel_luma - luminance;
            float midtone_weight = 4.0 * pixel_luma * (1.0 - pixel_luma);
            // Suppress broad halos around strong edges such as HUD text.
            float edge_weight = 1.0 - smoothstep(0.08, 0.28, abs(detail));
            float offset = clarity * midtone_weight * edge_weight * detail;
            float lo = min(color.r, min(color.g, color.b));
            float hi = max(color.r, max(color.g, color.b));
            color += clamp(offset, -lo, 1.0 - hi);
        }
        if (tone != 0 || bloom != 0) {
            color = saturate(color * exp2(-tone * (luminance - 0.4)));
            color += bloom * neighborhood * (1 - color);
        }
    }
    if (vibrance != 0) {
        float lo = min(color.r, min(color.g, color.b));
        float hi = max(color.r, max(color.g, color.b));
        float gray = dot(color, weights);
        float existing_saturation = (hi - lo) / max(hi, 1e-5);
        float gain = 1.0 + vibrance * (1.0 - existing_saturation);
        // Cap chroma gain before applying it to preserve hue near the gamut boundary.
        float dark_limit = gray / max(gray - lo, 1e-5);
        float light_limit = (1.0 - gray) / max(hi - gray, 1e-5);
        color = gray.xxx + (color - gray.xxx) * max(1.0, min(gain, min(dark_limit, light_limit)));
    }
    if (saturation != 0) {
        float gray = dot(color, weights);
        color = saturate(lerp(gray.xxx, color, 1 + saturation));
    }
    return float4(color, 1);
}
)";

LRESULT CALLBACK overlay_proc(HWND window, UINT message, WPARAM wparam, LPARAM lparam) {
    if (message == WM_MOUSEACTIVATE) return MA_NOACTIVATE;
    return DefWindowProcW(window, message, wparam, lparam);
}

class Overlay {
public:
    explicit Overlay(const RECT& bounds) {
        WNDCLASSW type{};
        type.lpfnWndProc = overlay_proc;
        type.hInstance = GetModuleHandleW(nullptr);
        type.lpszClassName = overlay_class;
        if (!RegisterClassW(&type) && GetLastError() != ERROR_CLASS_ALREADY_EXISTS) {
            winrt::throw_last_error();
        }
        window = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE,
            overlay_class, L"LumaTune", WS_POPUP, bounds.left, bounds.top,
            bounds.right - bounds.left, bounds.bottom - bounds.top,
            nullptr, nullptr, type.hInstance, nullptr);
        if (!window) winrt::throw_last_error();
        if (!SetLayeredWindowAttributes(window, 0, 255, LWA_ALPHA)) {
            auto error = GetLastError();
            DestroyWindow(window);
            winrt::throw_hresult(HRESULT_FROM_WIN32(error));
        }
    }

    ~Overlay() { DestroyWindow(window); }
    Overlay(const Overlay&) = delete;
    Overlay& operator=(const Overlay&) = delete;

    void place(const RECT& bounds) const {
        check_bool(SetWindowPos(window, HWND_TOPMOST, bounds.left, bounds.top,
            bounds.right - bounds.left, bounds.bottom - bounds.top,
            SWP_NOACTIVATE | SWP_SHOWWINDOW));
    }

    HWND window{};
};

struct Texture {
    com_ptr<ID3D11Texture2D> surface;
    com_ptr<ID3D11ShaderResourceView> view;
    com_ptr<ID3D11RenderTargetView> target;
};

struct Effects {
    float tone;
    float bloom;
    float saturation;
    float clarity;
    float vibrance;

    bool needs_blur() const { return tone != 0 || bloom != 0 || clarity != 0; }
};

struct ShaderSettings {
    float axis_x, axis_y;
    Effects effects;
    float padding{};
};
static_assert(sizeof(ShaderSettings) == 32);

com_ptr<ID3DBlob> compile(const char* entry, const char* stage) {
    com_ptr<ID3DBlob> binary;
    check_hresult(D3DCompile(shader_source, sizeof(shader_source) - 1, nullptr, nullptr, nullptr,
        entry, stage, D3DCOMPILE_OPTIMIZATION_LEVEL3, 0, binary.put(), nullptr));
    return binary;
}

struct Renderer {
    com_ptr<ID3D11Device> device;
    com_ptr<ID3D11DeviceContext> context;
    com_ptr<IDXGIFactory2> factory;
    com_ptr<IDXGISwapChain1> swap;
    com_ptr<ID3D11RenderTargetView> output;
    com_ptr<ID3D11VertexShader> vertex_shader;
    com_ptr<ID3D11PixelShader> blur_shader;
    com_ptr<ID3D11PixelShader> compose_shader;
    com_ptr<ID3D11SamplerState> sampler;
    com_ptr<ID3D11Buffer> constants;
    Texture input, horizontal, vertical;
    const UINT width, height, small_width, small_height;

    Renderer(UINT width, UINT height)
        : width(width), height(height), small_width(std::max(1u, width / 4)),
          small_height(std::max(1u, height / 4)) {
        check_hresult(D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr,
            D3D11_CREATE_DEVICE_BGRA_SUPPORT, nullptr, 0, D3D11_SDK_VERSION,
            device.put(), nullptr, context.put()));
        com_ptr<IDXGIAdapter> adapter;
        check_hresult(device.as<IDXGIDevice>()->GetAdapter(adapter.put()));
        check_hresult(adapter->GetParent(IID_PPV_ARGS(factory.put())));
    }

    bool is_sdr(HMONITOR monitor) const {
        const auto display_factory = factory.as<IDXGIFactory1>();
        com_ptr<IDXGIAdapter1> adapter;
        for (UINT a = 0; SUCCEEDED(display_factory->EnumAdapters1(a, adapter.put())); ++a, adapter = nullptr) {
            com_ptr<IDXGIOutput> display;
            for (UINT i = 0; SUCCEEDED(adapter->EnumOutputs(i, display.put())); ++i, display = nullptr) {
                auto info = display.try_as<IDXGIOutput6>();
                DXGI_OUTPUT_DESC1 desc{};
                if (info && SUCCEEDED(info->GetDesc1(&desc)) && desc.Monitor == monitor) {
                    return desc.ColorSpace == DXGI_COLOR_SPACE_RGB_FULL_G22_NONE_P709;
                }
            }
        }
        return false;
    }

    void initialize(HWND overlay, Effects effects) {
        check_hresult(factory->MakeWindowAssociation(overlay, DXGI_MWA_NO_ALT_ENTER));
        const DXGI_SWAP_CHAIN_DESC1 desc{width, height, DXGI_FORMAT_B8G8R8A8_UNORM, false, {1, 0},
            DXGI_USAGE_RENDER_TARGET_OUTPUT, 2, DXGI_SCALING_STRETCH, DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            DXGI_ALPHA_MODE_IGNORE, 0};
        check_hresult(factory->CreateSwapChainForHwnd(device.get(), overlay, &desc, nullptr, nullptr, swap.put()));
        com_ptr<ID3D11Texture2D> back_buffer;
        check_hresult(swap->GetBuffer(0, IID_PPV_ARGS(back_buffer.put())));
        check_hresult(device->CreateRenderTargetView(back_buffer.get(), nullptr, output.put()));
        initialize_pipeline(effects.needs_blur());
    }

    void initialize_pipeline(bool neighborhood) {
        auto vertex = compile("vertex", "vs_5_0");
        auto compose = compile("compose", "ps_5_0");
        check_hresult(device->CreateVertexShader(vertex->GetBufferPointer(), vertex->GetBufferSize(), nullptr, vertex_shader.put()));
        check_hresult(device->CreatePixelShader(compose->GetBufferPointer(), compose->GetBufferSize(), nullptr, compose_shader.put()));
        D3D11_SAMPLER_DESC sampling{};
        sampling.Filter = D3D11_FILTER_MIN_MAG_LINEAR_MIP_POINT;
        sampling.AddressU = sampling.AddressV = sampling.AddressW = D3D11_TEXTURE_ADDRESS_CLAMP;
        sampling.MaxLOD = D3D11_FLOAT32_MAX;
        check_hresult(device->CreateSamplerState(&sampling, sampler.put()));
        const D3D11_BUFFER_DESC constants_desc{sizeof(ShaderSettings), D3D11_USAGE_DEFAULT, D3D11_BIND_CONSTANT_BUFFER, 0, 0, 0};
        check_hresult(device->CreateBuffer(&constants_desc, nullptr, constants.put()));
        input = texture(width, height, false);
        if (neighborhood) {
            auto blur = compile("blur", "ps_5_0");
            check_hresult(device->CreatePixelShader(blur->GetBufferPointer(), blur->GetBufferSize(), nullptr, blur_shader.put()));
            horizontal = texture(small_width, small_height, true);
            vertical = texture(small_width, small_height, true);
        }

        context->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
        context->VSSetShader(vertex_shader.get(), nullptr, 0);
        auto sampler_ptr = sampler.get();
        context->PSSetSamplers(0, 1, &sampler_ptr);
        auto constants_ptr = constants.get();
        context->PSSetConstantBuffers(0, 1, &constants_ptr);
    }

    IDirect3DDevice capture_device() const {
        com_ptr<::IInspectable> wrapper;
        const auto dxgi = device.as<IDXGIDevice>();
        check_hresult(CreateDirect3D11DeviceFromDXGIDevice(dxgi.get(), wrapper.put()));
        return wrapper.as<IDirect3DDevice>();
    }

    bool draw(const Direct3D11CaptureFrame& frame, Effects effects) const {
        const auto captured = capture_texture(frame);
        context->CopyResource(input.surface.get(), captured.get());
        render_effects(output.get(), effects);
        const auto result = swap->Present(0, DXGI_PRESENT_DO_NOT_WAIT);
        if (result == DXGI_ERROR_WAS_STILL_DRAWING || result == DXGI_STATUS_OCCLUDED) return false;
        check_hresult(result);
        return true;
    }

    static com_ptr<ID3D11Texture2D> capture_texture(const Direct3D11CaptureFrame& frame) {
        const auto surface = frame.Surface().as<::Windows::Graphics::DirectX::Direct3D11::IDirect3DDxgiInterfaceAccess>();
        com_ptr<ID3D11Texture2D> result;
        check_hresult(surface->GetInterface(IID_PPV_ARGS(result.put())));
        return result;
    }

    void render_effects(ID3D11RenderTargetView* target, Effects effects) const {
        if (effects.needs_blur()) {
            paint(blur_shader.get(), horizontal.target.get(), small_width, small_height,
                input.view.get(), nullptr, {1.0f / small_width, 0, {}});
            paint(blur_shader.get(), vertical.target.get(), small_width, small_height,
                horizontal.view.get(), nullptr, {0, 1.0f / small_height, {}});
        }
        paint(compose_shader.get(), target, width, height,
            input.view.get(), vertical.view.get(), {0, 0, effects});
    }

    Texture texture(UINT w, UINT h, bool render_target) const {
        const D3D11_TEXTURE2D_DESC desc{w, h, 1, 1, DXGI_FORMAT_B8G8R8A8_UNORM, {1, 0}, D3D11_USAGE_DEFAULT,
            D3D11_BIND_SHADER_RESOURCE | (render_target ? D3D11_BIND_RENDER_TARGET : 0u), 0, 0};
        Texture result;
        check_hresult(device->CreateTexture2D(&desc, nullptr, result.surface.put()));
        check_hresult(device->CreateShaderResourceView(result.surface.get(), nullptr, result.view.put()));
        if (render_target) check_hresult(device->CreateRenderTargetView(result.surface.get(), nullptr, result.target.put()));
        return result;
    }

    void paint(ID3D11PixelShader* shader, ID3D11RenderTargetView* target, UINT w, UINT h,
        ID3D11ShaderResourceView* first, ID3D11ShaderResourceView* second,
        const ShaderSettings& settings) const {
        ID3D11ShaderResourceView* views[]{first, second};
        ID3D11ShaderResourceView* empty[]{nullptr, nullptr};
        const D3D11_VIEWPORT viewport{0, 0, static_cast<float>(w), static_cast<float>(h), 0, 1};
        context->OMSetRenderTargets(1, &target, nullptr);
        context->RSSetViewports(1, &viewport);
        context->UpdateSubresource(constants.get(), 0, nullptr, &settings, 0, 0);
        context->PSSetShader(shader, nullptr, 0);
        context->PSSetShaderResources(0, 2, views);
        context->Draw(3, 0);
        context->PSSetShaderResources(0, 2, empty);
        context->OMSetRenderTargets(0, nullptr, nullptr);
    }
};

bool target_is_active(HWND target, DWORD expected_pid) {
    DWORD pid = 0;
    return GetForegroundWindow() == target && !IsIconic(target)
        && GetWindowThreadProcessId(target, &pid) && pid == expected_pid;
}

RECT capture_bounds(HWND target) {
    RECT bounds{};
    if (FAILED(DwmGetWindowAttribute(target, DWMWA_EXTENDED_FRAME_BOUNDS, &bounds, sizeof(bounds)))) {
        check_bool(GetWindowRect(target, &bounds));
    }
    return bounds;
}

bool is_borderless(HWND target, const RECT& bounds) {
    if ((GetWindowLongPtrW(target, GWL_STYLE) & WS_CAPTION) != WS_CAPTION) return true;
    // Some frameless windows keep the caption style for shell integration.
    RECT client{};
    POINT origin{};
    return GetClientRect(target, &client) && ClientToScreen(target, &origin)
        && OffsetRect(&client, origin.x, origin.y) && EqualRect(&client, &bounds);
}

using Cancelled = bool (*)(const void*);

template <typename Captured>
struct CloseOnExit {
    Captured object;
    ~CloseOnExit() { try { object.Close(); } catch (...) {} }
};

struct HideOnExit {
    HWND window;
    ~HideOnExit() { ShowWindow(window, SW_HIDE); }
};

HRESULT process_window(HWND target, DWORD pid, Effects effects, const void* token, Cancelled cancelled) {
    if (!GraphicsCaptureSession::IsSupported()) return E_NOTIMPL;
    const auto active = [&] { return !cancelled(token) && target_is_active(target, pid); };
    if (!active()) return S_OK;
    auto bounds = capture_bounds(target);
    if (!is_borderless(target, bounds)) {
        return HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED);
    }

    const auto interop = winrt::get_activation_factory<GraphicsCaptureItem, IGraphicsCaptureItemInterop>();
    GraphicsCaptureItem item{nullptr};
    check_hresult(interop->CreateForWindow(target, winrt::guid_of<GraphicsCaptureItem>(), winrt::put_abi(item)));
    const auto size = item.Size();
    if (size.Width <= 0 || size.Height <= 0 || size.Width != bounds.right - bounds.left
        || size.Height != bounds.bottom - bounds.top) return HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED);
    auto monitor = MonitorFromWindow(target, MONITOR_DEFAULTTONULL);
    Overlay overlay(bounds);
    Renderer renderer(size.Width, size.Height);
    if (!renderer.is_sdr(monitor)) return HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED);
    renderer.initialize(overlay.window, effects);
    if (!active()) return S_OK;

    const auto closed = std::make_shared<std::atomic_bool>(false);
    const auto item_closed = item.Closed(winrt::auto_revoke, [closed](auto const&, auto const&) {
        closed->store(true);
    });
    const CloseOnExit pool{Direct3D11CaptureFramePool::CreateFreeThreaded(
        renderer.capture_device(), DirectXPixelFormat::B8G8R8A8UIntNormalized, 2, size)};
    const CloseOnExit session{pool.object.CreateCaptureSession(item)};
    const HideOnExit visibility{overlay.window};
    const auto cursor_control = session.object.try_as<IGraphicsCaptureSession2>();
    if (!cursor_control) return E_NOTIMPL;
    cursor_control.IsCursorCaptureEnabled(false);
    session.object.StartCapture();

    const auto startup_deadline = Clock::now() + 3s;
    auto next_display_check = Clock::now() + 1s;
    bool visible = false;
    while (active() && !closed->load()) {
        MSG message;
        while (PeekMessageW(&message, nullptr, 0, 0, PM_REMOVE)) {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        const auto next = capture_bounds(target);
        if (!is_borderless(target, next)) return HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED);
        if (size.Width != next.right - next.left || size.Height != next.bottom - next.top) return S_OK;
        const auto next_monitor = MonitorFromWindow(target, MONITOR_DEFAULTTONULL);
        const auto now = Clock::now();
        if (next_monitor != monitor || now >= next_display_check) {
            if (!renderer.is_sdr(next_monitor)) return HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED);
            next_display_check = now + 1s;
            monitor = next_monitor;
        }
        if (visible && !EqualRect(&bounds, &next) && active()) {
            overlay.place(next);
            bounds = next;
        }
        auto frame = pool.object.TryGetNextFrame();
        if (frame && active()) {
            const auto content = frame.ContentSize();
            if (content.Width != size.Width || content.Height != size.Height) return S_OK;
            if (renderer.draw(frame, effects) && active()) {
                if (!visible) overlay.place(next);
                bounds = next;
                visible = true;
            }
        }
        if (!visible && now >= startup_deadline) return HRESULT_FROM_WIN32(ERROR_TIMEOUT);
        if (!frame) MsgWaitForMultipleObjectsEx(0, nullptr, 4, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
    }
    return S_OK;
}

using PreviewSettings = std::uint64_t (*)(const void*, Effects*);
using PreviewPixels = void (*)(const void*, std::uint64_t, UINT, UINT,
    const std::uint8_t*, UINT, const std::uint8_t*, UINT);

struct ExcludePreviewWindow {
    HWND window;
    DWORD previous = WDA_NONE;

    explicit ExcludePreviewWindow(HWND window) : window(window) {
        if (window) {
            GetWindowDisplayAffinity(window, &previous);
            check_bool(SetWindowDisplayAffinity(window, WDA_EXCLUDEFROMCAPTURE));
        }
    }
    ~ExcludePreviewWindow() {
        if (window) SetWindowDisplayAffinity(window, previous);
    }
};

struct MappedPixels {
    ID3D11DeviceContext* context;
    ID3D11Texture2D* texture;
    D3D11_MAPPED_SUBRESOURCE data{};

    MappedPixels(ID3D11DeviceContext* context, ID3D11Texture2D* texture)
        : context(context), texture(texture) {
        check_hresult(context->Map(texture, 0, D3D11_MAP_READ, 0, &data));
    }
    ~MappedPixels() { context->Unmap(texture, 0); }
};

using winrt::Windows::Graphics::SizeInt32;

struct PreviewRenderer {
    Renderer renderer;
    Texture source, result;
    com_ptr<ID3D11Texture2D> before, after;

    static double scale(SizeInt32 size) {
        return std::min({1.0, 800.0 / size.Width, 480.0 / size.Height});
    }
    explicit PreviewRenderer(SizeInt32 size)
        : renderer(std::max(1u, static_cast<UINT>(size.Width * scale(size))),
              std::max(1u, static_cast<UINT>(size.Height * scale(size)))) {
        renderer.initialize_pipeline(true);
        // The downsampled input is also the original half of the preview.
        renderer.input = renderer.texture(renderer.width, renderer.height, true);
        source = renderer.texture(size.Width, size.Height, false);
        result = renderer.texture(renderer.width, renderer.height, true);
        D3D11_TEXTURE2D_DESC desc{renderer.width, renderer.height, 1, 1, DXGI_FORMAT_B8G8R8A8_UNORM,
            {1, 0}, D3D11_USAGE_STAGING, 0, D3D11_CPU_ACCESS_READ, 0};
        check_hresult(renderer.device->CreateTexture2D(&desc, nullptr, before.put()));
        check_hresult(renderer.device->CreateTexture2D(&desc, nullptr, after.put()));
    }

    void accept(const Direct3D11CaptureFrame& frame, SizeInt32 size) const {
        const auto captured = Renderer::capture_texture(frame);
        const D3D11_BOX crop{0, 0, 0, static_cast<UINT>(size.Width), static_cast<UINT>(size.Height), 1};
        renderer.context->CopySubresourceRegion(source.surface.get(), 0, 0, 0, 0,
            captured.get(), 0, &crop);
        renderer.paint(renderer.compose_shader.get(), renderer.input.target.get(),
            renderer.width, renderer.height, source.view.get(), nullptr, {});
    }

    void publish(Effects effects, std::uint64_t revision, const void* token, PreviewPixels pixels) const {
        renderer.render_effects(result.target.get(), effects);
        auto context = renderer.context.get();
        context->CopyResource(before.get(), renderer.input.surface.get());
        context->CopyResource(after.get(), result.surface.get());
        const MappedPixels left(context, before.get()), right(context, after.get());
        pixels(token, revision, renderer.width, renderer.height,
            static_cast<const std::uint8_t*>(left.data.pData), left.data.RowPitch,
            static_cast<const std::uint8_t*>(right.data.pData), right.data.RowPitch);
    }
};

HRESULT compare_capture(HWND window, DWORD pid, HMONITOR monitor, HWND editor,
    const void* token, Cancelled cancelled, PreviewSettings settings, PreviewPixels pixels) {
    if (!GraphicsCaptureSession::IsSupported()) return E_NOTIMPL;
    const auto valid = [&] {
        DWORD owner{};
        return !window || (GetWindowThreadProcessId(window, &owner) && owner == pid
            && IsWindowVisible(window) && !IsIconic(window));
    };
    if (cancelled(token)) return S_OK;
    if (!valid() || (!window && (!monitor || !editor))) return HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED);
    const ExcludePreviewWindow exclude(window ? nullptr : editor);
    const auto interop = winrt::get_activation_factory<GraphicsCaptureItem, IGraphicsCaptureItemInterop>();
    GraphicsCaptureItem item{nullptr};
    const auto iid = winrt::guid_of<GraphicsCaptureItem>();
    check_hresult(window ? interop->CreateForWindow(window, iid, winrt::put_abi(item))
        : interop->CreateForMonitor(monitor, iid, winrt::put_abi(item)));
    auto size = item.Size();
    if (size.Width <= 0 || size.Height <= 0) return HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED);
    const auto closed = std::make_shared<std::atomic_bool>(false);
    const auto item_closed = item.Closed(winrt::auto_revoke, [closed](auto const&, auto const&) {
        closed->store(true);
    });
    auto renderer = std::make_unique<PreviewRenderer>(size);
    const auto format = DirectXPixelFormat::B8G8R8A8UIntNormalized;
    const CloseOnExit pool{Direct3D11CaptureFramePool::CreateFreeThreaded(
        renderer->renderer.capture_device(), format, 2, size)};
    const CloseOnExit session{pool.object.CreateCaptureSession(item)};
    if (auto cursor = session.object.try_as<IGraphicsCaptureSession2>()) {
        cursor.IsCursorCaptureEnabled(false);
    }
    session.object.StartCapture();

    auto deadline = Clock::now() + 3s;
    bool has_frame = false;
    std::uint64_t last_revision = 0;
    while (!cancelled(token) && !closed->load() && valid()) {
        auto frame = pool.object.TryGetNextFrame();
        if (frame) {
            if (auto newer = pool.object.TryGetNextFrame()) {
                frame.Close();
                frame = std::move(newer);
            }
        }
        if (frame) {
            const auto next = frame.ContentSize();
            if (next.Width <= 0 || next.Height <= 0) return HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED);
            if (next.Width != size.Width || next.Height != size.Height) {
                frame.Close();
                size = next;
                renderer = std::make_unique<PreviewRenderer>(size);
                pool.object.Recreate(renderer->renderer.capture_device(), format, 2, size);
                has_frame = false;
                deadline = Clock::now() + 3s;
                continue;
            }
            renderer->accept(frame, size);
            frame.Close();
            has_frame = true;
        }
        Effects effects{};
        const auto revision = settings(token, &effects);
        if (has_frame && (frame || revision != last_revision)) {
            renderer->publish(effects, revision, token, pixels);
            last_revision = revision;
        }
        if (!has_frame && Clock::now() >= deadline) return HRESULT_FROM_WIN32(ERROR_TIMEOUT);
        Sleep(65);
    }
    return cancelled(token) ? S_OK : HRESULT_FROM_WIN32(ERROR_INVALID_WINDOW_HANDLE);
}
} // namespace

extern "C" std::int32_t lumatune_postprocess_run(
    std::uintptr_t window, std::uint32_t pid, float tone, float bloom, float saturation,
    float clarity, float vibrance, const void* token, Cancelled cancelled) noexcept {
    if (cancelled(token)) return S_OK;
    SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    const auto initialized = RoInitialize(RO_INIT_MULTITHREADED);
    if (FAILED(initialized)) return initialized;
    HRESULT result;
    try {
        result = process_window(reinterpret_cast<HWND>(window), pid,
            {tone, bloom, saturation, clarity, vibrance}, token, cancelled);
    } catch (const winrt::hresult_error& error) {
        result = error.code().value;
    } catch (...) {
        result = E_FAIL;
    }
    RoUninitialize();
    return result;
}

extern "C" std::int32_t lumatune_preview_run(
    std::uintptr_t window, std::uint32_t pid, std::uintptr_t monitor, std::uintptr_t editor,
    const void* token, Cancelled cancelled, PreviewSettings settings, PreviewPixels pixels) noexcept {
    SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    const auto initialized = RoInitialize(RO_INIT_MULTITHREADED);
    if (FAILED(initialized)) return initialized;
    HRESULT result;
    try {
        result = compare_capture(reinterpret_cast<HWND>(window), pid, reinterpret_cast<HMONITOR>(monitor),
            reinterpret_cast<HWND>(editor), token, cancelled, settings, pixels);
    } catch (const winrt::hresult_error& error) {
        result = error.code().value;
    } catch (...) {
        result = E_FAIL;
    }
    RoUninitialize();
    return result;
}
