# Module 11: Rust + Python Interop

**Accelerate Python with Rust Extensions**

## Overview

Learn to build high-performance Python extensions in Rust:
- PyO3 for Python bindings
- Performance-critical data processing
- Parallel processing with rayon
- NumPy integration
- Building and distributing packages
- Benchmarking and profiling

**Duration**: 2 weeks (20-25 hours)

## What You'll Build

```python
# Python code using your Rust extension

import my_rust_lib

# Fast data processing
data = list(range(1_000_000))
result = my_rust_lib.parallel_sum(data)
print(f"Sum: {result}")

# String processing
text = "hello world" * 100000
counts = my_rust_lib.word_count(text)
print(f"Word counts: {counts}")

# NumPy integration
import numpy as np
array = np.random.rand(1000, 1000)
result = my_rust_lib.matrix_multiply(array, array)

# Class-based API
processor = my_rust_lib.DataProcessor(chunk_size=1000)
processor.add_data(data)
stats = processor.get_statistics()
```

## Architecture

```
┌──────────────────────────────────────┐
│         Python Application           │
│      (your_script.py)                │
└────────────┬─────────────────────────┘
             │ import my_rust_lib
             │
┌────────────▼─────────────────────────┐
│         PyO3 Bindings                │
│   (Rust → Python FFI)                │
└────────────┬─────────────────────────┘
             │
┌────────────▼─────────────────────────┐
│      Rust Implementation             │
│   - Parallel processing (rayon)      │
│   - Unsafe optimizations             │
│   - Zero-copy where possible         │
└──────────────────────────────────────┘
```

## Key Components

### 1. Basic PyO3 Module

```rust
// lib.rs
use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn my_rust_lib(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(parallel_sum, m)?)?;
    m.add_function(wrap_pyfunction!(word_count, m)?)?;
    m.add_class::<DataProcessor>()?;
    Ok(())
}
```

**Cargo.toml:**
```toml
[package]
name = "my-rust-lib"
version = "0.1.0"
edition = "2021"

[lib]
name = "my_rust_lib"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.20", features = ["extension-module"] }
rayon = "1.7"
numpy = "0.20"
```

### 2. Parallel Data Processing

```rust
use pyo3::prelude::*;
use rayon::prelude::*;

/// Sum a list of integers in parallel
#[pyfunction]
fn parallel_sum(numbers: Vec<i64>) -> PyResult<i64> {
    let sum = numbers.par_iter().sum();
    Ok(sum)
}

/// Parallel filtering and mapping
#[pyfunction]
fn process_numbers(numbers: Vec<i64>, threshold: i64) -> PyResult<Vec<i64>> {
    let result: Vec<i64> = numbers
        .par_iter()
        .filter(|&&x| x > threshold)
        .map(|&x| x * 2)
        .collect();
    Ok(result)
}

/// Word count using parallel processing
#[pyfunction]
fn word_count(text: String) -> PyResult<std::collections::HashMap<String, usize>> {
    use std::collections::HashMap;
    use std::sync::Mutex;

    let counts = Mutex::new(HashMap::new());

    text.par_split_whitespace()
        .for_each(|word| {
            let word = word.to_lowercase();
            let mut map = counts.lock().unwrap();
            *map.entry(word).or_insert(0) += 1;
        });

    Ok(counts.into_inner().unwrap())
}
```

### 3. NumPy Integration

```rust
use pyo3::prelude::*;
use numpy::{PyArray1, PyArray2, PyReadonlyArray2};

/// Element-wise multiplication of two arrays
#[pyfunction]
fn array_multiply<'py>(
    py: Python<'py>,
    a: PyReadonlyArray2<f64>,
    b: PyReadonlyArray2<f64>,
) -> PyResult<&'py PyArray2<f64>> {
    let a = a.as_array();
    let b = b.as_array();

    // Check dimensions
    if a.shape() != b.shape() {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Arrays must have the same shape"
        ));
    }

    // Element-wise multiplication
    let result = &a * &b;

    // Convert back to Python
    Ok(PyArray2::from_owned_array(py, result))
}

/// Matrix multiplication using rayon for parallelism
#[pyfunction]
fn matrix_multiply<'py>(
    py: Python<'py>,
    a: PyReadonlyArray2<f64>,
    b: PyReadonlyArray2<f64>,
) -> PyResult<&'py PyArray2<f64>> {
    use ndarray::Array2;
    use rayon::prelude::*;

    let a = a.as_array();
    let b = b.as_array();

    let (m, n) = (a.nrows(), a.ncols());
    let (n2, p) = (b.nrows(), b.ncols());

    if n != n2 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Incompatible dimensions for matrix multiplication"
        ));
    }

    // Parallel matrix multiplication
    let mut result = Array2::zeros((m, p));

    result.axis_iter_mut(ndarray::Axis(0))
        .into_par_iter()
        .enumerate()
        .for_each(|(i, mut row)| {
            for j in 0..p {
                let sum: f64 = (0..n)
                    .map(|k| a[[i, k]] * b[[k, j]])
                    .sum();
                row[j] = sum;
            }
        });

    Ok(PyArray2::from_owned_array(py, result))
}

/// Fast statistical operations on NumPy arrays
#[pyfunction]
fn compute_stats<'py>(
    py: Python<'py>,
    arr: PyReadonlyArray1<f64>,
) -> PyResult<&'py PyDict> {
    let arr = arr.as_array();

    let mean = arr.mean().unwrap();
    let std = arr.std(1.0);
    let min = arr.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = arr.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    let dict = PyDict::new(py);
    dict.set_item("mean", mean)?;
    dict.set_item("std", std)?;
    dict.set_item("min", min)?;
    dict.set_item("max", max)?;

    Ok(dict)
}
```

### 4. Python Classes from Rust

```rust
use pyo3::prelude::*;
use std::collections::VecDeque;

#[pyclass]
struct DataProcessor {
    data: VecDeque<f64>,
    chunk_size: usize,
}

#[pymethods]
impl DataProcessor {
    #[new]
    fn new(chunk_size: usize) -> Self {
        DataProcessor {
            data: VecDeque::new(),
            chunk_size,
        }
    }

    /// Add data to the processor
    fn add_data(&mut self, values: Vec<f64>) {
        self.data.extend(values);
    }

    /// Process data in chunks
    fn process_chunks(&mut self) -> PyResult<Vec<f64>> {
        let mut results = Vec::new();

        while self.data.len() >= self.chunk_size {
            let chunk: Vec<f64> = self.data
                .drain(..self.chunk_size)
                .collect();

            // Process chunk (e.g., compute mean)
            let mean = chunk.iter().sum::<f64>() / chunk.len() as f64;
            results.push(mean);
        }

        Ok(results)
    }

    /// Get statistics about current data
    fn get_statistics(&self) -> PyResult<(usize, f64, f64)> {
        if self.data.is_empty() {
            return Ok((0, 0.0, 0.0));
        }

        let sum: f64 = self.data.iter().sum();
        let mean = sum / self.data.len() as f64;

        let variance = self.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / self.data.len() as f64;
        let std = variance.sqrt();

        Ok((self.data.len(), mean, std))
    }

    /// Clear all data
    fn clear(&mut self) {
        self.data.clear();
    }

    fn __len__(&self) -> usize {
        self.data.len()
    }

    fn __repr__(&self) -> String {
        format!("DataProcessor(size={}, chunk_size={})", self.data.len(), self.chunk_size)
    }
}
```

### 5. Error Handling

```rust
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

#[pyfunction]
fn divide(a: f64, b: f64) -> PyResult<f64> {
    if b == 0.0 {
        Err(PyValueError::new_err("Cannot divide by zero"))
    } else {
        Ok(a / b)
    }
}

// Custom exception types
pyo3::create_exception!(my_rust_lib, MyCustomError, pyo3::exceptions::PyException);

#[pyfunction]
fn risky_operation() -> PyResult<String> {
    Err(MyCustomError::new_err("Something went wrong!"))
}

// Converting Rust errors to Python
use std::fmt;

#[derive(Debug)]
struct MyError(String);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MyError: {}", self.0)
    }
}

impl std::error::Error for MyError {}

impl From<MyError> for PyErr {
    fn from(err: MyError) -> PyErr {
        PyValueError::new_err(err.to_string())
    }
}

#[pyfunction]
fn fallible_function() -> Result<String, MyError> {
    Err(MyError("Failed operation".to_string()))
}
```

### 6. Async Support

```rust
use pyo3::prelude::*;
use pyo3_asyncio::tokio::future_into_py;
use tokio::time::{sleep, Duration};

#[pyfunction]
fn async_sleep<'py>(py: Python<'py>, seconds: u64) -> PyResult<&'py PyAny> {
    future_into_py(py, async move {
        sleep(Duration::from_secs(seconds)).await;
        Ok(())
    })
}

#[pyfunction]
fn async_fetch_data<'py>(py: Python<'py>, url: String) -> PyResult<&'py PyAny> {
    future_into_py(py, async move {
        // Simulate async HTTP request
        sleep(Duration::from_millis(100)).await;
        Ok(format!("Data from {}", url))
    })
}
```

## Building and Packaging

### Development Workflow

```bash
# Create virtual environment
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install maturin (build tool for PyO3)
pip install maturin

# Develop mode (builds and installs in editable mode)
maturin develop

# Test in Python
python
>>> import my_rust_lib
>>> my_rust_lib.parallel_sum([1, 2, 3, 4, 5])
15
```

### Building Wheels

```bash
# Build wheel for current platform
maturin build --release

# Build for multiple Python versions
maturin build --release --interpreter python3.8 python3.9 python3.10 python3.11

# Build for distribution
maturin build --release --sdist
```

### pyproject.toml

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[project]
name = "my-rust-lib"
version = "0.1.0"
description = "High-performance data processing in Rust for Python"
requires-python = ">=3.8"
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Python :: 3.8",
    "Programming Language :: Python :: 3.9",
    "Programming Language :: Python :: 3.10",
    "Programming Language :: Python :: 3.11",
]
dependencies = ["numpy>=1.20"]

[tool.maturin]
python-source = "python"
module-name = "my_rust_lib"
```

## Performance Comparison

### Python Benchmark

```python
# benchmark.py
import time
import numpy as np
from my_rust_lib import parallel_sum, matrix_multiply

def benchmark_sum():
    data = list(range(10_000_000))

    # Pure Python
    start = time.time()
    result = sum(data)
    python_time = time.time() - start

    # Rust
    start = time.time()
    result = parallel_sum(data)
    rust_time = time.time() - start

    print(f"Python sum: {python_time:.4f}s")
    print(f"Rust sum:   {rust_time:.4f}s")
    print(f"Speedup:    {python_time / rust_time:.2f}x")

def benchmark_matrix():
    size = 1000
    a = np.random.rand(size, size)
    b = np.random.rand(size, size)

    # NumPy
    start = time.time()
    result = np.dot(a, b)
    numpy_time = time.time() - start

    # Rust
    start = time.time()
    result = matrix_multiply(a, b)
    rust_time = time.time() - start

    print(f"NumPy matmul: {numpy_time:.4f}s")
    print(f"Rust matmul:  {rust_time:.4f}s")
    print(f"Speedup:      {numpy_time / rust_time:.2f}x")

if __name__ == "__main__":
    benchmark_sum()
    print()
    benchmark_matrix()
```

## Testing

### Rust Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_sum() {
        let numbers = vec![1, 2, 3, 4, 5];
        let result = parallel_sum(numbers).unwrap();
        assert_eq!(result, 15);
    }
}
```

### Python Tests

```python
# tests/test_my_rust_lib.py
import pytest
import numpy as np
from my_rust_lib import (
    parallel_sum,
    word_count,
    matrix_multiply,
    DataProcessor,
)

def test_parallel_sum():
    result = parallel_sum([1, 2, 3, 4, 5])
    assert result == 15

def test_word_count():
    text = "hello world hello rust"
    counts = word_count(text)
    assert counts["hello"] == 2
    assert counts["world"] == 1
    assert counts["rust"] == 1

def test_matrix_multiply():
    a = np.array([[1.0, 2.0], [3.0, 4.0]])
    b = np.array([[5.0, 6.0], [7.0, 8.0]])
    result = matrix_multiply(a, b)
    expected = np.dot(a, b)
    np.testing.assert_array_almost_equal(result, expected)

def test_data_processor():
    processor = DataProcessor(chunk_size=3)
    processor.add_data([1.0, 2.0, 3.0, 4.0, 5.0, 6.0])

    assert len(processor) == 6

    results = processor.process_chunks()
    assert len(results) == 2  # Two chunks of 3
    assert len(processor) == 0  # All data processed

def test_error_handling():
    from my_rust_lib import divide
    with pytest.raises(ValueError, match="Cannot divide by zero"):
        divide(10.0, 0.0)
```

## Implementation Roadmap

### Phase 1: Setup & Basic Functions (Days 1-2)
- Set up PyO3 project
- Create basic module with simple functions
- Build and import in Python
- Write initial tests

**Success criteria:**
- Can build and import Rust module in Python
- Basic functions work
- Tests pass

### Phase 2: Data Processing (Days 3-5)
- Implement parallel processing with rayon
- List/vector conversions
- String processing
- Performance benchmarks

**Success criteria:**
- Parallel functions faster than pure Python
- Handles large datasets
- Memory efficient

### Phase 3: NumPy Integration (Days 6-9)
- NumPy array conversions
- Matrix operations
- Statistical functions
- Zero-copy optimizations

**Success criteria:**
- Works with NumPy arrays
- Competitive with NumPy performance
- No unnecessary copies

### Phase 4: Classes & Advanced Features (Days 10-13)
- Python classes from Rust structs
- Stateful objects
- Error handling
- Python special methods

**Success criteria:**
- Idiomatic Python API
- Good error messages
- Proper resource cleanup

### Phase 5: Packaging & Distribution (Days 14-17)
- Build wheels for multiple platforms
- Set up CI/CD
- Documentation
- Publish to PyPI (optional)

**Success criteria:**
- Wheels build on Linux, macOS, Windows
- Documentation complete
- Easy installation

### Phase 6: Real-World Use Case (Days 18-20)
- Build complete application
- Comprehensive benchmarks
- Profiling and optimization
- Production deployment guide

## Use Cases

### 1. Data Processing Pipeline

```python
# data_pipeline.py
import pandas as pd
from my_rust_lib import DataProcessor

# Read large CSV
df = pd.read_csv("large_dataset.csv")

# Process with Rust
processor = DataProcessor(chunk_size=10000)
processor.add_data(df['values'].tolist())

# Get results
stats = processor.get_statistics()
print(f"Mean: {stats[1]}, Std: {stats[2]}")
```

### 2. Image Processing

```python
import numpy as np
from PIL import Image
from my_rust_lib import process_image

# Load image
img = Image.open("photo.jpg")
img_array = np.array(img)

# Process in Rust (e.g., blur, edge detection)
processed = process_image(img_array)

# Save result
Image.fromarray(processed).save("output.jpg")
```

### 3. Scientific Computing

```python
import numpy as np
from my_rust_lib import monte_carlo_pi

# Estimate π using Monte Carlo method
n_samples = 100_000_000
pi_estimate = monte_carlo_pi(n_samples)
print(f"π ≈ {pi_estimate}")
```

## Performance Tips

### 1. Minimize Python/Rust Boundary Crossings
```rust
// ❌ Bad: Call from Python in loop
#[pyfunction]
fn process_single(x: f64) -> f64 {
    x * 2.0
}

// ✅ Good: Process entire batch in Rust
#[pyfunction]
fn process_batch(xs: Vec<f64>) -> Vec<f64> {
    xs.into_iter().map(|x| x * 2.0).collect()
}
```

### 2. Use Zero-Copy When Possible
```rust
// Use PyReadonlyArray for read-only access (zero-copy)
#[pyfunction]
fn sum_array(arr: PyReadonlyArray1<f64>) -> f64 {
    arr.as_array().sum()
}
```

### 3. Release GIL for CPU-Bound Work
```rust
use pyo3::prelude::*;

#[pyfunction]
fn cpu_intensive(py: Python, data: Vec<f64>) -> PyResult<f64> {
    // Release GIL while doing computation
    py.allow_threads(|| {
        // Expensive computation
        data.iter().map(|x| x.sin()).sum()
    })
}
```

## Success Criteria

- ✅ Build Python extensions with PyO3
- ✅ Parallel processing with rayon
- ✅ NumPy array integration
- ✅ Python classes from Rust
- ✅ Proper error handling
- ✅ Build wheels for distribution
- ✅ 10x+ speedup over pure Python
- ✅ Comprehensive tests

## Resources

**PyO3:**
- [PyO3 User Guide](https://pyo3.rs/)
- [PyO3 Examples](https://github.com/PyO3/pyo3/tree/main/examples)
- [Maturin Documentation](https://github.com/PyO3/maturin)

**Python Packaging:**
- [Python Packaging Guide](https://packaging.python.org/)
- [Building Binary Extensions](https://setuptools.pypa.io/en/latest/userguide/ext_modules.html)

**Performance:**
- [Rayon Documentation](https://docs.rs/rayon/)
- [ndarray Documentation](https://docs.rs/ndarray/)
- [numpy crate](https://docs.rs/numpy/)

**Real-World Examples:**
- [Polars](https://github.com/pola-rs/polars) - DataFrame library
- [Ruff](https://github.com/astral-sh/ruff) - Fast Python linter
- [Cryptography](https://github.com/pyca/cryptography) - Uses Rust for crypto

## Next Module

[Module 12: Rust + TypeScript Interop →](../module-12-typescript-interop/)
