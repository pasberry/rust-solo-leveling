use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// Console logging from WASM
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Simple greeting function
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Compute fibonacci number (iterative, more efficient)
#[wasm_bindgen]
pub fn fibonacci(n: u32) -> u64 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }

    let mut prev = 0u64;
    let mut curr = 1u64;

    for _ in 2..=n {
        let next = prev + curr;
        prev = curr;
        curr = next;
    }

    curr
}

/// Sum an array of numbers
#[wasm_bindgen]
pub fn sum_array(numbers: &[f64]) -> f64 {
    numbers.iter().sum()
}

/// Process text: count words, characters, and lines
#[wasm_bindgen]
pub fn analyze_text(text: &str) -> JsValue {
    #[derive(Serialize)]
    struct TextStats {
        words: usize,
        characters: usize,
        lines: usize,
        unique_words: usize,
    }

    let words: Vec<&str> = text.split_whitespace().collect();
    let unique_words: std::collections::HashSet<&str> = words.iter().copied().collect();

    let stats = TextStats {
        words: words.len(),
        characters: text.chars().count(),
        lines: text.lines().count(),
        unique_words: unique_words.len(),
    };

    serde_wasm_bindgen::to_value(&stats).unwrap()
}

/// DataProcessor class - demonstrates stateful WASM structs
#[wasm_bindgen]
pub struct DataProcessor {
    data: Vec<f64>,
}

#[wasm_bindgen]
impl DataProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> DataProcessor {
        DataProcessor { data: Vec::new() }
    }

    pub fn add_data(&mut self, values: Vec<f64>) {
        self.data.extend(values);
    }

    pub fn mean(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.data.iter().sum::<f64>() / self.data.len() as f64
    }

    pub fn median(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        let mut sorted = self.data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }

    pub fn std_dev(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        let mean = self.mean();
        let variance: f64 = self
            .data
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / self.data.len() as f64;
        variance.sqrt()
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Complex data structures using serde
#[derive(Serialize, Deserialize)]
pub struct User {
    name: String,
    email: String,
    age: u32,
}

/// Process user data
#[wasm_bindgen]
pub fn process_user(user_js: JsValue) -> Result<JsValue, JsValue> {
    let user: User = serde_wasm_bindgen::from_value(user_js)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse user: {}", e)))?;

    // Process user
    let result = format!(
        "{} ({}) is {} years old",
        user.name, user.email, user.age
    );

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
}

/// Hash a string (simple hash function)
#[wasm_bindgen]
pub fn hash_string(input: &str) -> u64 {
    let mut hash: u64 = 0;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

/// Check if a number is prime
#[wasm_bindgen]
pub fn is_prime(n: u64) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }

    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

/// Find prime numbers up to n
#[wasm_bindgen]
pub fn primes_up_to(n: u32) -> Vec<u32> {
    let mut primes = Vec::new();
    for i in 2..=n {
        if is_prime(i as u64) {
            primes.push(i);
        }
    }
    primes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(10), 55);
        assert_eq!(fibonacci(20), 6765);
    }

    #[test]
    fn test_sum_array() {
        assert_eq!(sum_array(&[1.0, 2.0, 3.0, 4.0, 5.0]), 15.0);
        assert_eq!(sum_array(&[]), 0.0);
    }

    #[test]
    fn test_data_processor() {
        let mut processor = DataProcessor::new();
        assert!(processor.is_empty());

        processor.add_data(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(processor.len(), 5);
        assert_eq!(processor.mean(), 3.0);
        assert_eq!(processor.median(), 3.0);

        processor.clear();
        assert!(processor.is_empty());
    }

    #[test]
    fn test_is_prime() {
        assert!(!is_prime(0));
        assert!(!is_prime(1));
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(4));
        assert!(is_prime(5));
        assert!(is_prime(7));
        assert!(!is_prime(9));
        assert!(is_prime(11));
    }

    #[test]
    fn test_primes_up_to() {
        let primes = primes_up_to(20);
        assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19]);
    }

    #[test]
    fn test_hash_string() {
        let hash1 = hash_string("hello");
        let hash2 = hash_string("hello");
        let hash3 = hash_string("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
