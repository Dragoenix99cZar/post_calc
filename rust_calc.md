# calc_engine

To build a high-performance scientific calculator for a web application, Rust compiled to WebAssembly (Wasm) is an excellent choice. It provides near-native execution speed, memory safety, and seamless integration with a frontend framework via wasm-bindgen.
Below is the complete port of your Python calculator architecture into a production-ready Rust library structured for WebAssembly compilation.

------------------------------
## 🛠️ 1. Project Setup (Cargo.toml)
Create a new Rust library (cargo new calc_engine --lib) and replace your configuration file with the following dependencies.

```toml
[package]
name = "calc_engine"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
wasm-bindgen = "0.2"
regex = "1.10"
lazy_static = "1.4"
```

------------------------------
## 🦀 2. The Core Rust Implementation (src/lib.rs)
This code implements your context-aware tokenizer, Shunting-Yard parser, postfix execution flow, and configuration tables entirely in Rust.

```rust
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use regex::Regex;
use std::f64::consts::{PI, E};
// Define constants map globallyconst TAU: f64 = PI * 2.0;
// Helper to determine operator attributes cleanly
fn get_precedence(op: &str) -> i32 {
    match op {
        "deg2rad" | "rad2deg" | "neg" => 5,
        "sin" | "cos" | "tan" | "sqrt" | "ln" | "log10" | "log2" | "exp" | "atan2" => 4,
        "^" => 3,
        "*" | "/" => 2,
        "+" | "-" => 1,
        _ => 0,
    }
}
fn is_right_associative(op: &str) -> bool {
    op == "^" || op == "neg"
}
fn get_constant(constant: &str) -> Option<f64> {
    match constant.to_lowercase().as_str() {
        "pi" => Some(PI),
        "e" => Some(E),
        "tau" => Some(TAU),
        _ => None,
    }
}
// Custom check to validate if token is a valid numeric scalar string
fn is_numeric(token: &str) -> bool {
    if token.parse::<f64>().is_ok() {
        return true;
    }
    false
}
fn tokenize(expression: &str) -> Vec<String> {
    // Matches: identifiers/words, integers/floats, and math operators/brackets
    let re = Regex::new(r"[a-zA-Z][a-zA-Z0-9]*|\d+(?:\.\d+)?|[\+\-\*\/\^\(\),]").unwrap();
    let raw_tokens: Vec<String> = re.find_iter(expression)
        .map(|m| m.as_str().to_string())
        .collect();

    let mut refined_tokens = Vec::new();
    let mut i = 0;

    while i < raw_tokens.len() {
        let token = &raw_tokens[i];

        if token == "-" {
            // Determine structural context for unary validation
            let is_unary = refined_tokens.is_empty() || 
                refined_tokens.last().map_or(false, |last| {
                    last == "(" || last == "," || get_precedence(last) > 0
                });

            if is_unary {
                // Peek ahead to check if a numeric component immediately follows
                if i + 1 < raw_tokens.len() && is_numeric(&raw_tokens[i + 1]) {
                    refined_tokens.push(format!("-{}", raw_tokens[i + 1]));
                    i += 2;
                    continue;
                } else {
                    refined_tokens.push("neg".to_string());
                    i += 1;
                    continue;
                }
            }
        }
        refined_tokens.push(token.clone());
        i += 1;
    }
    refined_tokens
}
fn infix_to_postfix(infix_expression: &str) -> Vec<String> {
    let tokens = tokenize(infix_expression);
    let mut output_queue = Vec::new();
    let mut operator_stack = Vec::new();

    for token in tokens {
        if is_numeric(&token) || get_constant(&token).is_some() {
            output_queue.push(token.to_lowercase());
        } else if get_precedence(&token) > 0 {
            while let Some(top_op) = operator_stack.last() {
                if get_precedence(top_op) > get_precedence(&token) || 
                   (get_precedence(top_op) == get_precedence(&token) && !is_right_associative(&token)) {
                    output_queue.push(operator_stack.pop().unwrap());
                } else {
                    break;
                }
            }
            operator_stack.push(token);
        } else if token == "," {
            while let Some(top) = operator_stack.last() {
                if top != "(" {
                    output_queue.push(operator_stack.pop().unwrap());
                } else {
                    break;
                }
            }
        } else if token == "(" {
            operator_stack.push(token);
        } else if token == ")" {
            while let Some(top) = operator_stack.last() {
                if top != "(" {
                    output_queue.push(operator_stack.pop().unwrap());
                } else {
                    break;
                }
            }
            if operator_stack.last().map_or(false, |top| top == "(") {
                operator_stack.pop();
            }
        }
    }

    while let Some(op) = operator_stack.pop() {
        output_queue.push(op);
    }

    output_queue
}
fn evaluate_postfix(tokens: Vec<String>) -> Result<f64, String> {
    let mut stack: Vec<f64> = Vec::new();

    for token in tokens {
        if is_numeric(&token) {
            let val = token.parse::<f64>().map_err(|_| "Failed to parse number".to_string())?;
            stack.push(val);
        } else if let Some(constant_val) = get_constant(&token) {
            stack.push(constant_val);
        } else {
            // Check Unary functions first
            match token.as_str() {
                "deg2rad" | "rad2deg" | "sin" | "cos" | "tan" | "sqrt" | "ln" | "log10" | "log2" | "exp" | "neg" => {
                    let val = stack.pop().ok_ok_or_else(|| "Stack underflow for unary operator".to_string())?;
                    let result = match token.as_str() {
                        "deg2rad" => val.to_radians(),
                        "rad2deg" => val.to_degrees(),
                        "sin"     => val.sin(),
                        "cos"     => val.cos(),
                        "tan"     => val.tan(),
                        "sqrt"    => val.sqrt(),
                        "ln"      => val.ln(),
                        "log10"   => val.log10(),
                        "log2"    => val.log2(),
                        "exp"     => val.exp(),
                        "neg"     => -val,
                        _ => unreachable!(),
                    };
                    stack.push(result);
                }
                // Check Binary Functions next
                "+" | "-" | "*" | "/" | "^" | "atan2" => {
                    let b = stack.pop().ok_or_else(|| "Stack underflow for binary operator".to_string())?;
                    let a = stack.pop().ok_or_else(|| "Stack underflow for binary operator".to_string())?;
                    let result = match token.as_str() {
                        "+" => a + b,
                        "-" => a - b,
                        "*" => a * b,
                        "/" => a / b,
                        "^" => a.powf(b),
                        "atan2" => a.atan2(b),
                        _ => unreachable!(),
                    };
                    stack.push(result);
                }
                _ => return Err(format!("Unknown token encountered: {}", token)),
            }
        }
    }

    stack.pop().ok_or_else(|| "Evaluation finished with an empty stack".to_string())
}
/// Expose calculation function directly to WebAssembly Javascript Runtime
#[wasm_bindgen]pub fn calculate(expression: &str) -> Result<f64, JsValue> {
    let postfix = infix_to_postfix(expression);
    evaluate_postfix(postfix).map_err(|err| JsValue::from_str(&err))
}
```

------------------------------
## 🧪 3. Complete Rust Test Suite (src/lib.rs under implementation)
Add this test module block to the bottom of your src/lib.rs file to replicate your comprehensive verification suite. You can run these using cargo test.

```rust
#[cfg(test)]mod tests {
    use super::*;

    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if ($x - $y).abs() > $d {
                panic!("Assertion failed: {} != {} (delta > {})", $x, $y, $d);
            }
        };
    }

    #[test]
    fn run_all_precedence_and_coverage_tests() {
        let delta = 1e-9;

        // Operators & Associativity
        assert_delta!(calculate("3 + 5").unwrap(), 8.0, delta);
        assert_delta!(calculate("10 - 4").unwrap(), 6.0, delta);
        assert_delta!(calculate("4 * 2.5").unwrap(), 10.0, delta);
        assert_delta!(calculate("9 / 2").unwrap(), 4.5, delta);
        assert_delta!(calculate("2 ^ 3").unwrap(), 8.0, delta);
        assert_delta!(calculate("2 ^ 3 ^ 2").unwrap(), 512.0, delta); 
        assert_delta!(calculate("10 - 4 - 2").unwrap(), 4.0, delta);

        // Precedence Mixes
        assert_delta!(calculate("3 + 5 * 2").unwrap(), 13.0, delta);
        assert_delta!(calculate("5 * 2 ^ 3").unwrap(), 40.0, delta);
        assert_delta!(calculate("( 3 + 5 ) * 2").unwrap(), 16.0, delta);
        assert_delta!(calculate("2 ^ ( 1 + 2 )").unwrap(), 8.0, delta);

        // Constants
        assert_delta!(calculate("pi").unwrap(), PI, delta);
        assert_delta!(calculate("e").unwrap(), E, delta);
        assert_delta!(calculate("tau").unwrap(), TAU, delta);

        // Unary
        assert_delta!(calculate("deg2rad(180)").unwrap(), PI, delta);
        assert_delta!(calculate("rad2deg(pi)").unwrap(), 180.0, delta);
        assert_delta!(calculate("sin(pi / 2)").unwrap(), 1.0, delta);
        assert_delta!(calculate("cos(0)").unwrap(), 1.0, delta);
        assert_delta!(calculate("sqrt(9)").unwrap(), 3.0, delta);
        assert_delta!(calculate("log10(100)").unwrap(), 2.0, delta);

        // Negatives
        assert_delta!(calculate("-1").unwrap(), -1.0, delta);
        assert_delta!(calculate("atan2(0, -1)").unwrap(), PI, delta);

        // Complex Multi-Function Mix
        assert_delta!(calculate("3 + sin(deg2rad(30)) * 2 ^ 3").unwrap(), 7.0, delta);
        assert_delta!(calculate("log2(8) + log10(100) * sqrt(rad2deg(atan2(1, 1)))").unwrap(), 16.41640786499874, delta);
    }
}
```

------------------------------
## 🚀 4. Compiling and Serving to Your Web App
To compile your Rust engine into a package that can be imported directly into a Web Browser (Vanilla JS, React, Vue, Next.js, etc.):

   1. Install wasm-pack:
```console
   > cargo install wasm-pack
```   
   2. Build the target package for web applications:
```console
   > wasm-pack build --target web
```
   
This generates a ./pkg directory containing JavaScript glue files alongside the optimized binary .wasm engine. You can import and use it in your web project like this:

```js
import init, { calculate } from './pkg/calc_engine.js';
async function run() {
    // Initialize WebAssembly environment
    await init();
    
    try {
        const expression = "log2(8) + log10(100) * sqrt(rad2deg(atan2(1, 1)))";
        const result = calculate(expression);
        console.log("Calculation Result:", result); // Outputs: 16.41640786499874
    } catch (error) {
        console.error("Calculation Error:", error);
    }
}
run();
```
---
# **`Give sample index file, also handle error in rust`**

Here is how you handle errors properly in your Rust calculation engine and clean up your WebAssembly interface, followed by a complete standalone HTML/JavaScript sample index file to test it right in your browser.
------------------------------
## 🦀 1. Enhanced Rust Error Handling (src/lib.rs)
To ensure your web app doesn't crash on invalid input (like syntax errors, mismatched parentheses, or division by zero), we can update evaluate_postfix to use Rust's standard Result<f64, String> pattern and cleanly forward error messages to JavaScript.
Replace the execution functions in your src/lib.rs with this error-hardened implementation:


```rust
use wasm_bindgen::prelude::*;
// ... (keep the `tokenize` and `infix_to_postfix` functions from the previous block)
fn evaluate_postfix(tokens: Vec<String>) -> Result<f64, String> {
    let mut stack: Vec<f64> = Vec::new();

    for token in tokens {
        if is_numeric(&token) {
            let val = token.parse::<f64>().map_err(|_| "Invalid number format".to_string())?;
            stack.push(val);
        } else if let Some(constant_val) = get_constant(&token) {
            stack.push(constant_val);
        } else {
            match token.as_str() {
                // --- Unary Functions ---
                "deg2rad" | "rad2deg" | "sin" | "cos" | "tan" | "sqrt" | "ln" | "log10" | "log2" | "exp" | "neg" => {
                    let val = stack.pop().ok_or_else(|| "Malformed expression: Stack underflow".to_string())?;
                    
                    // Specific domain error checks
                    if token == "sqrt" && val < 0.0 {
                        return Err("Math Error: Square root of a negative number".to_string());
                    }
                    if (token == "ln" || token == "log10" || token == "log2") && val <= 0.0 {
                        return Err("Math Error: Logarithm of a non-positive number".to_string());
                    }

                    let result = match token.as_str() {
                        "deg2rad" => val.to_radians(),
                        "rad2deg" => val.to_degrees(),
                        "sin"     => val.sin(),
                        "cos"     => val.cos(),
                        "tan"     => val.tan(),
                        "sqrt"    => val.sqrt(),
                        "ln"      => val.ln(),
                        "log10"   => val.log10(),
                        "log2"    => val.log2(),
                        "exp"     => val.exp(),
                        "neg"     => -val,
                        _ => unreachable!(),
                    };
                    stack.push(result);
                }
                // --- Binary Operators ---
                "+" | "-" | "*" | "/" | "^" | "atan2" => {
                    let b = stack.pop().ok_or_else(|| "Malformed expression: Stack underflow".to_string())?;
                    let a = stack.pop().ok_or_else(|| "Malformed expression: Stack underflow".to_string())?;
                    
                    // Division by zero check
                    if token == "/" && b == 0.0 {
                        return Err("Math Error: Division by Zero".to_string());
                    }

                    let result = match token.as_str() {
                        "+" => a + b,
                        "-" => a - b,
                        "*" => a * b,
                        "/" => a / b,
                        "^" => a.powf(b),
                        "atan2" => a.atan2(b),
                        _ => unreachable!(),
                    };
                    stack.push(result);
                }
                _ => return Err(format!("Syntax Error: Unknown token '{}'", token)),
            }
        }
    }

    if stack.len() != 1 {
        return Err("Syntax Error: The expression has too many standalone values".to_string());
    }

    stack.pop().ok_or_else(|| "Syntax Error: Empty evaluation stack".to_string())
}
/// Main entry point for JavaScript. 
/// Converts errors into an explicit string exception that JS can `catch`.
#[wasm_bindgen]pub fn calculate(expression: &str) -> Result<f64, JsValue> {
    let postfix = infix_to_postfix(expression);
    evaluate_postfix(postfix).map_err(|err| JsValue::from_str(&err))
}
```

------------------------------
## 🌐 2. Web Application Interface (index.html)
After building your Rust code with wasm-pack build --target web, place this index.html file right outside or next to your generated ./pkg directory. It sets up a minimal, functional UI to demonstrate normal operations alongside catchable error alerts.

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Rust WebAssembly Scientific Calculator</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: #f4f5f6;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
        }
        .calc-container {
            background: white;
            padding: 24px;
            border-radius: 12px;
            box-shadow: 0 4px 20px rgba(0,0,0,0.08);
            width: 400px;
        }
        input {
            width: 100%;
            padding: 12px;
            font-size: 16px;
            border: 1px solid #ccc;
            border-radius: 6px;
            box-sizing: border-box;
            margin-bottom: 12px;
        }
        button {
            width: 100%;
            padding: 12px;
            background: #0070f3;
            color: white;
            border: none;
            font-size: 16px;
            border-radius: 6px;
            cursor: pointer;
            font-weight: 600;
        }
        button:hover { background: #0060df; }
        .result-box {
            margin-top: 16px;
            padding: 12px;
            border-radius: 6px;
            font-weight: bold;
            display: none;
        }
        .success { background: #e6f6ec; color: #137333; border: 1px solid #c2e7cc; }
        .error { background: #fce8e6; color: #c5221f; border: 1px solid #fad2cf; }
    </style>
</head>
<body>

<div class="calc-container">
    <h2>Scientific Engine</h2>
    <input type="text" id="expression" value="log2(8) + log10(100) * sqrt(rad2deg(atan2(1, 1)))" placeholder="Enter math formula...">
    <button id="calc-btn">Evaluate Expression</button>
    <div id="display-output" class="result-box"></div>
</div>

<script type="module">
    // Import the compiled WebAssembly loader and function wrapper 
    import init, { calculate } from './pkg/calc_engine.js';

    async function initializeCalculator() {
        // Initialize the Wasm binary
        await init();

        const inputEl = document.getElementById('expression');
        const buttonEl = document.getElementById('calc-btn');
        const outputEl = document.getElementById('display-output');

        buttonEl.addEventListener('click', () => {
            const expression = inputEl.value;
            outputEl.style.display = 'block';

            try {
                // Call the exported Rust function directly
                const outputResult = calculate(expression);
                
                outputEl.className = 'result-box success';
                outputEl.innerText = `Result: ${outputResult}`;
            } catch (errorMessage) {
                // Rust errors catch directly here as string exceptions
                outputEl.className = 'result-box error';
                outputEl.innerText = `Error: ${errorMessage}`;
            }
        });
    }

    initializeCalculator();
</script>

</body>
</html>
```

## 🛠️ How to View and Test This Locally
Because WebAssembly files must be loaded via modern JavaScript modules over HTTP, you cannot double-click the HTML file to run it locally. You need to serve it using a local HTTP server:

   1. Open your terminal in the directory containing your index.html and the ./pkg folder.
   2. Run a simple local server using Python or Node.js:
   * Python: python -m http.server 8080
      * Node/NPX: npx serve
   3. Open http://localhost:8080 in your web browser.

Try testing standard formulas alongside error conditions like 5 / 0 or sqrt(-9) to see the catch block handle the Rust panic-safe strings cleanly in real time!
Would you like to extend the parsing logic next to support implicit multiplication (e.g., automatically evaluating 2pi as 2 * pi), or should we design a calculation history state mechanism to easily pass old output values into new operations?



