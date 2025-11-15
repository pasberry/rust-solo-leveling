use pyo3::prelude::*;
use pyo3::types::PyDict;

/// A Rust struct exposed to Python
#[pyclass]
#[derive(Clone)]
struct Calculator {
    #[pyo3(get, set)]
    memory: f64,
}

#[pymethods]
impl Calculator {
    #[new]
    fn new() -> Self {
        Calculator { memory: 0.0 }
    }

    fn add(&mut self, a: f64, b: f64) -> f64 {
        let result = a + b;
        self.memory = result;
        result
    }

    fn subtract(&mut self, a: f64, b: f64) -> f64 {
        let result = a - b;
        self.memory = result;
        result
    }

    fn multiply(&mut self, a: f64, b: f64) -> f64 {
        let result = a * b;
        self.memory = result;
        result
    }

    fn divide(&mut self, a: f64, b: f64) -> PyResult<f64> {
        if b == 0.0 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Division by zero",
            ));
        }
        let result = a / b;
        self.memory = result;
        Ok(result)
    }

    fn clear_memory(&mut self) {
        self.memory = 0.0;
    }
}

/// Process a list of numbers - demonstrates working with Python collections
#[pyfunction]
fn process_numbers(numbers: Vec<f64>) -> PyResult<(f64, f64, f64)> {
    if numbers.is_empty() {
        return Ok((0.0, 0.0, 0.0));
    }

    let sum: f64 = numbers.iter().sum();
    let min = numbers.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = numbers.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    Ok((sum, min, max))
}

/// Fibonacci sequence generator
#[pyfunction]
fn fibonacci(n: usize) -> Vec<u64> {
    let mut fib = vec![0, 1];
    for i in 2..n {
        let next = fib[i - 1] + fib[i - 2];
        fib.push(next);
    }
    fib.truncate(n);
    fib
}

/// String processing function
#[pyfunction]
fn reverse_string(s: String) -> String {
    s.chars().rev().collect()
}

/// Count word frequency
#[pyfunction]
fn word_frequency(text: String) -> PyResult<Py<PyDict>> {
    Python::with_gil(|py| {
        let dict = PyDict::new(py);

        for word in text.to_lowercase().split_whitespace() {
            let cleaned: String = word.chars()
                .filter(|c| c.is_alphanumeric())
                .collect();

            if !cleaned.is_empty() {
                let count: i32 = dict.get_item(&cleaned)?
                    .and_then(|v| v.extract().ok())
                    .unwrap_or(0);
                dict.set_item(cleaned, count + 1)?;
            }
        }

        Ok(dict.into())
    })
}

/// Data processing class
#[pyclass]
struct DataProcessor {
    data: Vec<f64>,
}

#[pymethods]
impl DataProcessor {
    #[new]
    fn new(data: Vec<f64>) -> Self {
        DataProcessor { data }
    }

    fn mean(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.data.iter().sum::<f64>() / self.data.len() as f64
    }

    fn median(&self) -> f64 {
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

    fn std_dev(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        let mean = self.mean();
        let variance: f64 = self.data.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / self.data.len() as f64;
        variance.sqrt()
    }

    fn filter_outliers(&self, std_devs: f64) -> Vec<f64> {
        let mean = self.mean();
        let std = self.std_dev();
        let threshold = std * std_devs;

        self.data.iter()
            .filter(|&&x| (x - mean).abs() <= threshold)
            .copied()
            .collect()
    }
}

/// Python module definition
#[pymodule]
fn rust_py_lib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Calculator>()?;
    m.add_class::<DataProcessor>()?;
    m.add_function(wrap_pyfunction!(process_numbers, m)?)?;
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    m.add_function(wrap_pyfunction!(reverse_string, m)?)?;
    m.add_function(wrap_pyfunction!(word_frequency, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator() {
        let mut calc = Calculator::new();
        assert_eq!(calc.add(5.0, 3.0), 8.0);
        assert_eq!(calc.memory, 8.0);
        assert_eq!(calc.multiply(2.0, 4.0), 8.0);
    }

    #[test]
    fn test_fibonacci() {
        let fib = fibonacci(10);
        assert_eq!(fib, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }

    #[test]
    fn test_data_processor() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let processor = DataProcessor::new(data);
        assert_eq!(processor.mean(), 3.0);
        assert_eq!(processor.median(), 3.0);
    }
}
