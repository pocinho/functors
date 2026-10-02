# **Phase 1 Milestone Plan — MVU Core + Rendering Engine**

## **Goal of Phase 1**
Establish the foundational architecture of Functors:

- deterministic MVU engine  
- GPU‑accelerated rendering pipeline  
- basic text buffer  
- minimal UI loop  
- project structure + documentation  
- first visible prototype  

This phase is about building the *heart* of Functors.

---

# **Milestone 1 — Project Foundations**
### **Objectives**
- Initialize repo (`main`, `develop`)
- Add documentation structure (mdBook + API docs)
- Add `.gitignore`, `README.md`, `CONTRIBUTING.md`
- Define crate layout

### **Deliverables**
- `functors/` repo with docs committed  
- `src/` structured into modules:
  ```
  src/
    main.rs
    app/
    mvu/
    rendering/
    text/
    platform/
  ```

### **Success Criteria**
Functors builds, runs, and has a documented architecture.

---

# **Milestone 2 — MVU Core**
### **Objectives**
Build the MVU engine:

- `Model` struct  
- `Message` enum  
- `update(model, msg)` function  
- `Command` abstraction for async tasks  
- `Subscription` abstraction for external events  
- `View` trait (platform‑agnostic)

### **Deliverables**
- `mvu/` module with:
  ```
  model.rs
  message.rs
  update.rs
  command.rs
  subscription.rs
  view.rs
  ```

### **Success Criteria**
You can run a simple MVU counter app inside Functors.

---

# **Milestone 3 — Event Loop + Platform Shell**
### **Objectives**
Integrate MVU with a platform loop:

- Use **winit** for window + input  
- Create the main event loop  
- Connect winit events → MVU messages  
- Connect MVU view → rendering pipeline

### **Deliverables**
- `app/` module:
  ```
  app.rs
  event_loop.rs
  platform.rs
  ```

### **Success Criteria**
A window opens, receives input, and updates MVU state.

---

# **Milestone 4 — Rendering Pipeline (WGPU)**
### **Objectives**
Set up GPU rendering:

- Initialize WGPU  
- Create swap chain  
- Create render pass  
- Clear screen  
- Draw basic shapes  
- Render text via Vello or SkiaSafe (choose one)

### **Deliverables**
- `rendering/` module:
  ```
  renderer.rs
  gpu.rs
  primitives.rs
  text.rs
  ```

### **Success Criteria**
Functors renders:

- background  
- a rectangle  
- “Hello Functors” text  

All driven by MVU.

---

# **Milestone 5 — Text Buffer + Cursor**
### **Objectives**
Implement minimal editor functionality:

- text buffer  
- cursor movement  
- basic input handling  
- rendering text in a grid  
- MVU messages for editing

### **Deliverables**
- `text/` module:
  ```
  buffer.rs
  cursor.rs
  layout.rs
  ```

### **Success Criteria**
You can type text into Functors and see it rendered.

---

# **Milestone 6 — Layout Engine (Basic)**
### **Objectives**
Create a simple layout system:

- vertical stack layout  
- padding + margins  
- text block layout  
- cell-like containers (foundation for notebooks)

### **Deliverables**
- `rendering/layout.rs`

### **Success Criteria**
Functors can render:

- header  
- text area  
- footer  

All MVU-driven.

---

# **Milestone 7 — First Prototype**
### **Objectives**
Combine everything:

- MVU  
- rendering  
- text buffer  
- layout  
- event loop  

### **Deliverables**
A working prototype:

- opens a window  
- renders text  
- updates via MVU  
- handles input  
- displays basic UI structure  

### **Success Criteria**
You can type into Functors and see the UI update smoothly.

---

# **Milestone 8 — Documentation + Release**
### **Objectives**
Document Phase 1:

- architecture  
- MVU engine  
- rendering pipeline  
- text buffer  
- layout system  
- roadmap for Phase 2 (Notebook)

### **Deliverables**
- mdBook updated  
- API docs updated  
- `v0.1.0` tag on `main`

### **Success Criteria**
Functors has a stable, documented foundation.

---

# **Phase 1 Summary**

| Milestone | Description | Outcome |
|----------|-------------|---------|
| 1 | Repo + docs | Project skeleton |
| 2 | MVU core | Deterministic state engine |
| 3 | Event loop | Platform integration |
| 4 | Rendering | GPU pipeline |
| 5 | Text buffer | Basic editor |
| 6 | Layout | UI structure |
| 7 | Prototype | First visible app |
| 8 | Docs + release | v0.1.0 |
