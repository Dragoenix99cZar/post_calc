# post_calc
Infix-to-Postfix Calculator

```text
╔══════════════════════════════════════════════════════════════════════════╗
║                        RUST + WASM SCIENTIFIC ENGINE                     ║
║                       High-Performance Math Calculator                   ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### 1. VISION
  To build a blazingly fast, memory-safe, cross-platform scientific calculation 
  engine that runs natively on Windows[cite: 1] and seamlessly inside any web browser via 
  WebAssembly (Wasm)[cite: 1] with zero runtime dependencies.


### 2. IDEATION
  * Why Rust? Near-native execution speed, ironclad safety guarantees, and robust 
    compile-time type checking[cite: 1].
  * Why Wasm? Bridges the gap between systems-level performance and web UI responsiveness.
  * Architecture Strategy: Write core math logic once (Tokenizer -> Shunting-Yard Parser 
    -> Postfix Evaluator)[cite: 1] and dual-target it for native CLI usage and browser execution.


### 3. ARCHITECTURE

```
     ┌────────────────────────────────────────────────────────┐
     │                     User Interfaces                    │
     ├──────────────────────────┬─────────────────────────────┤
     │  Web UI (index.html)     │  Native CLI (src/main.rs)   │
     │  - Wasm Loader           │  - REPL Interactive Mode    │
     │  - Keypad & Grid Controls│  - Direct Argument Parsing  │
     └─────────────┬────────────┴──────────────┬──────────────┘
                   │                           │
                   ▼                           ▼
     ┌────────────────────────────────────────────────────────┐
     │                 Rust Calculation Engine                │
     ├────────────────────────────────────────────────────────┤
     │  1. Tokenizer          (Regex pattern matching)        │
     │  2. Shunting-Yard      (Infix to Postfix RPN)          │
     │  3. Postfix Evaluator  (Stack-based math + domains)    │
     └─────────────┬──────────────────────────┬───────────────┘
                   │ (wasm-bindgen)           │ (Native Result)
                   ▼                          ▼
            [ Browser Window ]          [ Windows Terminal ]
```

### 4. PROJECT SETUP
  1. Initialize Cargo Library & CLI components:

```console
     > cargo new calc_engine --lib
```
     
  2. Configure dependencies in `Cargo.toml:

```toml
     [dependencies]
     wasm-bindgen = "0.2"[cite: 1]
     regex = "1.10"[cite: 1]
     lazy_static = "1.4"[cite: 1]
```

### 3. RUN & BUILD COMMANDS

  • Run Unit Tests (Rust core logic verification):
```console  
> cargo test
```

  • Run CLI Interactive REPL (Windows):
```console
> cargo run
```

  • Evaluate CLI Expression directly:
```console
> cargo run -- "log2(8) + log10(100)"
```


• Build Standalone Native Release Binary:

```console
> cargo build --release
```    
    *(Binary outputs to: target/release/post_calc.exe)*

  • Build WebAssembly package for the Web App:
```console 
> wasm-pack build --target web
```

  • Serve Web App Locally (Python server):
```console
> python -m http.server 8080
```
    (Open http://localhost:8080 in your browser)