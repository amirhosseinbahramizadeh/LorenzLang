
```markdown
<div align="center">

<img src="https://upload.wikimedia.org/wikipedia/commons/4/45/Lorenz_attractor_yb.svg" alt="Lorenz Attractor" width="400"/>

# 🦋 Lorenz
### The World's First Deterministic Chaos Programming Language

[![Rust](https://img.shields.io/badge/Built_with-Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/Version-0.1.0-blue.svg?style=for-the-badge)]()

<i>Variables are no longer static scalars. They are probability clouds.</i>

[Getting Started](#-installation) • [Documentation](#-syntax--concepts) • [VS Code Extension](#-vs-code-extension) • [Architecture](#-under-the-hood)

</div>

---

## 🌪️ The Paradigm Shift

For 70 years, programming languages have assumed a deterministic universe. In standard languages, `x = x + 1` implies absolute precision forever. But the physical world—fluid dynamics, quantum mechanics, global supply chains, and orbital mechanics—doesn't work like that. It is governed by **Chaos Theory**.

Traditional languages simulate chaotic systems by running Monte Carlo simulations thousands of times, burning massive compute power. **Lorenz natively speaks chaos.** 

In Lorenz, every variable is a `ChaoticVar`: a mathematical structure containing a `mean` (expected value), a `variance` (uncertainty), and a `sensitivity_map` (how it reacts to other variables). When you propagate them through time, the Lorenz VM uses Lyapunov exponents to exponentially expand their uncertainty based on the mathematical formula:

> **σ²(t) = σ²₀ · e^(2λt)**

### Standard Languages vs. Lorenz

<div align="center">

| Python (Blind to the future) | Lorenz (Embraces reality) |
| :--- | :--- |
| `pressure = 101.0` <br> `temp = 20.0` <br> `result = pressure + temp` | `let pressure = chaotic(101.0, 0.1)` <br> `let temp = chaotic(20.0, 0.5)` <br> `collapse(propagate(pressure + temp, 1.0))` |
| <i>Result: 121.0 (Always)</i> | <i>Result: 121.34, 121.88, 120.11 (Dynamic)</i> |

</div>

---

## 🛡️ The Butterfly Profiler

Lorenz doesn't just execute code; it predicts its mathematical demise. 

Before the Lorenz VM executes a single instruction, the **Butterfly Profiler** performs Static Chaos Analysis on the Abstract Syntax Tree (AST). If the mathematical model indicates that a variable's variance will exceed a safe threshold during time propagation, **the compiler halts with a `BUTTERFLY ANOMALY`.**

```text
❌ Lorenz Parse Error: BUTTERFLY ANOMALY
   Propagating 'sensor' for 60.0s expands variance to 4405.2, exceeding safe limit of 100.0
```
*Stop the simulation before the system explodes.*

---

## 🚀 Installation

Requires [Rust](https://rustup.rs/) to be installed.

```bash
git clone https://github.com/YOUR_USERNAME/lorenz-lang.git
cd lorenz-lang
cargo build --release
```
The binary will be located at `target/release/lorenz` (or `lorenz.exe` on Windows). Add it to your system's `PATH` for global access.

---

## 💻 Syntax & Concepts

Create a file named `simulation.lz`:

```lorenz
// Define chaotic variables: chaotic(mean, initial_variance)
let base_temp = chaotic(100.0, 0.5)
let coolant_flow = chaotic(20.0, 0.1)

// Propagate system forward in time and collapse the probability wave
collapse(propagate(base_temp + coolant_flow, 3.0))
```

Execute it through the Lorenz VM:

```bash
$ lorenz run simulation.lz

[Lorenz State] Mean: 120.500, Variance: 0.840, StdDev: 0.916
Lorenz Output: 121.32
```
*(Running this command multiple times will yield slightly different outputs, accurately reflecting the collapse of a probability distribution).*

---

## 🖥️ VS Code Extension

For the ultimate developer experience, install the official [Lorenz VS Code Extension](https://github.com/amirhosseinbahramizadeh/lorenz-vscode). 

Features:
✅ Beautiful Syntax Highlighting for `.lz` files
✅ Built-in `Run` Button (`Ctrl+Shift+L`)
✅ Direct integration with the Lorenz VM terminal

---

## ⚙️ Under The Hood

Lorenz is built from absolute scratch with zero runtime bloat. No LLVM, no external frameworks.

* **Lexer & Parser:** A hand-written recursive-descent parser for `.lz` files.
* **Butterfly Profiler:** An AST walker that calculates Lyapunov variance growth before execution.
* **Compiler:** Transforms the AST into a compact, custom Bytecode.
* **Lorenz VM:** A stack-based virtual machine where the stack doesn't hold integers—it holds `ChaoticVar` structs, natively computing covariance matrices during arithmetic.

---

## 📜 Why "Lorenz"?

Named after [Edward Lorenz](https://en.wikipedia.org/wiki/Edward_Lorenz), the mathematician and meteorologist who pioneered chaos theory and coined the term "Butterfly Effect". His equations proved that a butterfly flapping its wings in Brazil could set off a tornado in Texas. Now, you can code that butterfly.

<div align="center">
Made with ❤️ by [Amirhossein Bahramizadeh] • Released under the [MIT License](LICENSE)
</div>
```
