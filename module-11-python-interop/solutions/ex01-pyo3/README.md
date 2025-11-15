# Module 11: Python Interop with PyO3

Python bindings for Rust code using PyO3.

## Features

- **Calculator**: Stateful calculator with memory
- **Fibonacci**: Fast Fibonacci sequence generator
- **Data Processing**: Statistical functions (mean, median, std dev, outlier filtering)
- **String Processing**: String manipulation functions
- **Word Frequency**: Text analysis

## Build

```bash
# Install maturin for building Python wheels
pip install maturin

# Build the extension
maturin develop

# Or build a wheel
maturin build --release
```

## Usage

```python
import rust_py_lib

# Calculator
calc = rust_py_lib.Calculator()
result = calc.add(5, 3)
print(calc.memory)

# Fibonacci
fib = rust_py_lib.fibonacci(10)

# Data processing
processor = rust_py_lib.DataProcessor([1.0, 2.0, 3.0, 4.0, 5.0])
print(processor.mean())
```

## Run Example

```bash
python python/example.py
```

## Tests

```bash
cargo test
```
