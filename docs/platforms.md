# Platform Architecture & Deployment Strategy

Functors is designed as a **cross‑platform MVU-first editor and computational notebook**, capable of running on desktop, mobile, and eventually web environments. This document outlines the architectural strategy for each platform, the rendering pipeline, and the integration layers required to maintain a unified codebase.

---

## 1. Overview

Functors follows a **single-core architecture**:

- **MVU Engine** — pure, deterministic, platform-agnostic  
- **Rendering Pipeline** — GPU-accelerated, cross-platform  
- **Notebook Runtime** — WASM/native execution  
- **Agent Layer** — local or remote LLMs  
- **Workspace Model** — filesystem-backed project structure  

All platforms share these layers. Only the **platform shell** changes.

---

# 2. Platform Targets

Functors supports (or will support) the following platforms:

| Platform | Status | Rendering Backend | Shell / Windowing | Notes |
|---------|--------|-------------------|--------------------|-------|
| **Windows** | Primary | WGPU (DX12) | Winit | Full editor experience |
| **macOS** | Primary | WGPU (Metal) | Winit | Full editor experience |
| **Linux** | Primary | WGPU (Vulkan) | Winit | Full editor experience |
| **Android** | Planned | WGPU (Vulkan) | NativeActivity + Winit | Mobile editor + notebook |
| **iOS** | Future | WGPU (Metal) | UIKit shell + Winit (experimental) | Requires custom integration |
| **Web (WASM)** | Future | WGPU (WebGPU) | Browser canvas | Limited editor, full notebook |

---

# 3. Shared Architecture

All platforms share the same core:

```
+------------------------------+
|          Platform Shell      |
|  (Winit / NativeActivity)    |
+------------------------------+
              |
              v
+------------------------------+
|           MVU Engine         |
|  Model / Update / View       |
+------------------------------+
              |
              v
+------------------------------+
|            Renderer          |
|   WGPU + Vello/Skia          |
+------------------------------+
              |
              v
+------------------------------+
|        Notebook Runtime      |
|   WASM / Native Execution    |
+------------------------------+
              |
              v
+------------------------------+
|         Agent Layer          |
|   Local LLM / Remote API     |
+------------------------------+
```

This ensures:

- deterministic behavior  
- identical rendering across platforms  
- unified plugin system  
- shared workspace model  
- consistent agent capabilities  

---

# 4. Desktop Platforms (Windows, macOS, Linux)

### Rendering
- **WGPU**  
  - DX12 (Windows)  
  - Metal (macOS)  
  - Vulkan (Linux)

### Windowing
- **Winit**  
  - Keyboard + mouse input  
  - IME support  
  - Clipboard  
  - DPI scaling  

### Text & UI Rendering
- **Vello** (preferred)  
- or **SkiaSafe** (fallback)

### Notebook Execution
- Native runtimes  
- WASM runtimes  
- Visualization cells via GPU surfaces  

### Agent Integration
- Local LLMs (llama.cpp, ONNX, QNN)  
- Remote inference endpoints  
- Copilot integration  

---

# 5. Android Platform

Android support uses the same rendering pipeline as desktop, with a different shell.

## 5.1 Architecture

```
Android NativeActivity
        |
        v
ndk-glue + winit (Android backend)
        |
        v
WGPU (Vulkan)
        |
        v
Vello / Skia
        |
        v
MVU Engine
```

## 5.2 Components

### Shell
- **NativeActivity** (no Java UI required)
- **ndk-glue** for lifecycle
- **winit** for input + window

### Rendering
- **WGPU (Vulkan)**  
- **Vello** for text + vector graphics

### Optional JNI Integration
Only if needed:
- file pickers  
- notifications  
- Android storage APIs  
- system dialogs  

### Notebook Execution
- WASM runtime (safe, portable)  
- Native runtimes (optional, sandboxed)  

### Agent Integration
- Local LLMs (via Rust)  
- Remote inference endpoints  

## 5.3 Why this stack is ideal

- MVU maps perfectly to Android’s event loop  
- GPU rendering ensures smooth text + notebook cells  
- No dependency on Jetpack Compose or Flutter  
- Unified codebase with desktop  

---

# 6. iOS Platform (Future)

iOS support mirrors Android but requires:

- **Metal backend** (WGPU)  
- **UIKit shell**  
- **custom winit backend** (experimental)  
- **strict sandboxing** for runtimes  

This is feasible but requires additional engineering.

---

# 7. Web Platform (Future)

Web support is possible via:

- **WASM**  
- **WebGPU**  
- **winit (web backend)**  
- **Vello (WebGPU)**  

Limitations:

- No native filesystem  
- Limited local LLM support  
- Reduced editor performance  

But notebook + agent workflows will work well.

---

# 8. Plugin System Across Platforms

Plugins are designed to be platform-agnostic:

- language support  
- syntax highlighting  
- cell types  
- agent integrations  
- commands  
- renderers  

Plugins communicate through a stable Rust API and can be compiled for:

- desktop  
- Android  
- WASM (future)

---

# 9. Summary

Functors uses a **unified Rust architecture** across all platforms:

- MVU core  
- WGPU renderer  
- Vello/Skia text engine  
- Winit windowing  
- WASM/native notebook runtimes  
- agent protocol  

This ensures:

- consistent behavior  
- shared codebase  
- high performance  
- extensibility  
- long-term maintainability  

Android support fits naturally into this architecture with minimal divergence.
