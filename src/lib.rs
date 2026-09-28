use regex::Regex;
use std::f64::consts::{E, PI};
use wasm_bindgen::prelude::*;

// Define constants map globally
const TAU: f64 = PI * 2.0;

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
    token.parse::<f64>().is_ok()
}

fn tokenize(expression: &str) -> Vec<String> {
    let re = Regex::new(r"[a-zA-Z][a-zA-Z0-9]*|\d+(?:\.\d+)?|[\+\-\*\/\^\(\),]").unwrap();
    let raw_tokens: Vec<String> = re
        .find_iter(expression)
        .map(|m| m.as_str().to_string())
        .collect();

    // Explicitly type-annotate as Vec<String> to avoid unsized str inference
    let mut refined_tokens: Vec<String> = Vec::new();
    let mut i = 0;

    while i < raw_tokens.len() {
        let token = &raw_tokens[i];

        if token == "-" {
            let is_unary = refined_tokens.is_empty()
                || refined_tokens.last().map_or(false, |last| {
                    last == "(" || last == "," || get_precedence(last) > 0
                });

            if is_unary {
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
    let mut output_queue: Vec<String> = Vec::new();
    // Explicitly type-annotate operator_stack as Vec<String>
    let mut operator_stack: Vec<String> = Vec::new();

    for token in tokens {
        if is_numeric(&token) || get_constant(&token).is_some() {
            output_queue.push(token.to_lowercase());
        } else if get_precedence(&token) > 0 {
            while let Some(top_op) = operator_stack.last() {
                if get_precedence(top_op) > get_precedence(&token)
                    || (get_precedence(top_op) == get_precedence(&token)
                        && !is_right_associative(&token))
                {
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
            let val = token
                .parse::<f64>()
                .map_err(|_| "Invalid number format".to_string())?;
            stack.push(val);
        } else if let Some(constant_val) = get_constant(&token) {
            stack.push(constant_val);
        } else {
            match token.as_str() {
                "deg2rad" | "rad2deg" | "sin" | "cos" | "tan" | "sqrt" | "ln" | "log10"
                | "log2" | "exp" | "neg" => {
                    // Fixed typo: changed ok_ok_or_else to ok_or_else
                    let val = stack
                        .pop()
                        .ok_or_else(|| "Malformed expression: Stack underflow".to_string())?;

                    if token == "sqrt" && val < 0.0 {
                        return Err("Math Error: Square root of a negative number".to_string());
                    }
                    if (token == "ln" || token == "log10" || token == "log2") && val <= 0.0 {
                        return Err("Math Error: Logarithm of a non-positive number".to_string());
                    }

                    let result = match token.as_str() {
                        "deg2rad" => val.to_radians(),
                        "rad2deg" => val.to_degrees(),
                        "sin" => val.sin(),
                        "cos" => val.cos(),
                        "tan" => val.tan(),
                        "sqrt" => val.sqrt(),
                        "ln" => val.ln(),
                        "log10" => val.log10(),
                        "log2" => val.log2(),
                        "exp" => val.exp(),
                        "neg" => -val,
                        _ => unreachable!(),
                    };
                    stack.push(result);
                }
                "+" | "-" | "*" | "/" | "^" | "atan2" => {
                    let b = stack
                        .pop()
                        .ok_or_else(|| "Malformed expression: Stack underflow".to_string())?;
                    let a = stack
                        .pop()
                        .ok_or_else(|| "Malformed expression: Stack underflow".to_string())?;

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

    stack
        .pop()
        .ok_or_else(|| "Syntax Error: Empty evaluation stack".to_string())
}

/// Expose calculation function directly to WebAssembly Javascript Runtime
#[wasm_bindgen]
pub fn calculate(expression: &str) -> Result<f64, JsValue> {
    let postfix = infix_to_postfix(expression);
    evaluate_postfix(postfix).map_err(|err| JsValue::from_str(&err))
}

/// Native entry point for command-line and non-Wasm environments
pub fn evaluate_expression(expression: &str) -> Result<f64, String> {
    let postfix = infix_to_postfix(expression);
    evaluate_postfix(postfix)
}
