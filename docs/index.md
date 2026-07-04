# Lorenz: The Chaos Theory Programming Language

**The world's first programming language with native support for Chaos Theory and the Butterfly Effect.**

---

## What is Lorenz?

Lorenz is a revolutionary programming language designed for **chaotic systems simulation**. Unlike traditional languages that deal with deterministic values, Lorenz variables represent **probability clouds** — distributions that evolve and expand over time due to sensitivity to initial conditions.

Named after Edward Lorenz, the father of chaos theory, this language brings mathematical elegance to the unpredictability of complex systems.

---

## Variables as Probability Clouds

In traditional programming:

```python
temperature = 20.0  # A single, fixed number
```

In Lorenz:

```
let temperature = chaotic(20.0, 0.25)  # Mean: 20.0, Variance: 0.25
```

This represents a **probability distribution** — not a single value, but a cloud of possible values that expands over time. The `0.25` variance means our uncertainty grows exponentially, just like in real chaotic systems.

---

## Python vs Lorenz

| Feature | Python | Lorenz |
|---------|--------|--------|
| Variables | Deterministic values | Probability clouds |
| Time evolution | Manual implementation | Built-in `propagate()` |
| Uncertainty tracking | Not supported | Native variance/sensitivity |
| Collapse to value | N/A | Built-in `collapse()` |
| Chaos detection | Manual | Automatic Butterfly Profiler |
| Domain | General purpose | Chaotic systems simulation |

---

## Basic Lorenz Syntax

```
// Define chaotic variables with initial uncertainty
let pressure = chaotic(101.0, 0.1)
let temperature = chaotic(20.0, 0.5)

// Propagate forward in time (variance grows exponentially)
let evolved_pressure = propagate(pressure, 1.0)

// Add variables (covariance-aware)
let system = evolved_pressure + temperature

// Collapse to a single measurement
let result = collapse(system)

// Output: 121.61075678309253
```

---

## The Butterfly Effect in Code

The **Butterfly Profiler** automatically detects when small uncertainties could lead to massive prediction errors — the hallmark of chaotic systems.

```rust
// This will be caught by the profiler:
let x = chaotic(10.0, 100.0)  // High initial uncertainty
collapse(propagate(x, 100.0))  // Variance explodes!

// Error: BUTTERFLY ANOMALY detected
// Variance exceeds safe threshold
```

---

## Key Features

- **100% Safe Rust** — Memory safe, no crashes
- **Stack-based VM** — Fast bytecode execution
- **Covariance-aware arithmetic** — Variables can be correlated
- **Butterfly Profiler** — Static analysis for chaos detection
- **Cloud Ready** — HTTP API for web services
- **VS Code Extension** — Syntax highlighting and instant execution

---

## Getting Started

### Installation

```bash
# Clone the repository
git clone https://github.com/amirhosseinbahramizadeh/Programming-Lang-PD
cd Programming-Lang-PD

# Build
cargo build --release

# Run a program
./target/release/lorenz model.lz
```

### VS Code Extension

Install the `.vsix` extension for syntax highlighting and one-click execution.

### Web API

```bash
# Start the server
lorenz serve

# Evaluate code via HTTP
curl -X POST http://localhost:8080/evaluate \
  -H "Content-Type: application/json" \
  -d '{"code": "let a = chaotic(10.0, 0.1)\n collapse(propagate(a, 2.0))"}'
```

---

## Use Cases

- **Climate Modeling** — Track uncertainty in weather predictions
- **Financial Systems** — Model market volatility
- **Military Radar** — Handle sensor noise in targeting systems
- **Epidemiology** — Predict disease spread with uncertainty
- **Quantum Computing** — Simulate probabilistic systems

---

## Learn More

- [Military Simulation Example](military.md) — See Lorenz in action for air defense systems

---

*Built with Rust. Designed for chaos.*