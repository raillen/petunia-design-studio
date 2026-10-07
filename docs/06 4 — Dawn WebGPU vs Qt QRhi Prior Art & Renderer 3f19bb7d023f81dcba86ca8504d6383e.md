# 06.4 — Dawn/WebGPU vs Qt QRhi Prior Art & Renderer Risk Notes

# Dawn

Dawn implements WebGPU natively over APIs such as D3D12, Metal and Vulkan and includes state validation and shader translation infrastructure. This makes it attractive as a modern explicit GPU abstraction for Petunia-owned rendering/computation.

# Dawn advantages to test

Portable modern resource/pipeline model; WGSL/Tint ecosystem; compute suitability; explicit validation; separation from Qt.

# Dawn costs to test

Petunia would still need substantial 2D path tessellation, text/image/effect rendering machinery or pair Dawn with other libraries.

# QRhi

Qt 6.12 documents QRhi as an accelerated graphics abstraction with Vulkan, OpenGL ES, D3D11/D3D12, Metal and Null implementations.

# Critical QRhi risk

Qt documentation also states QRhi is in the Qt GUI private API family with limited compatibility guarantees and requires GuiPrivate linkage. Therefore Petunia must not let QRhi types leak into canonical renderer/domain contracts.

# Suitable QRhi roles

Renderer spike candidate; presentation/swapchain integration; perhaps production backend only if version-lock/maintenance costs prove acceptable.

# Sources

- [Dawn architecture overview](https://dawn.googlesource.com/dawn/+/HEAD/docs/dawn/overview.md)
- [Qt 6.12 QRhi documentation](https://doc.qt.io/qt-6.12/qrhi.html)

# Petunia consequence

G020/G021 explicitly measure backend capability, integration cost and upgrade risk; RenderScene remains toolkit-neutral regardless of winner.