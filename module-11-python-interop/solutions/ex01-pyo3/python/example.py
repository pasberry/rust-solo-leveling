"""
Example Python script demonstrating Rust-Python interop
"""

import rust_py_lib

def main():
    # Test Calculator
    print("=== Calculator ===")
    calc = rust_py_lib.Calculator()
    print(f"5 + 3 = {calc.add(5, 3)}")
    print(f"Memory: {calc.memory}")
    print(f"10 * 4 = {calc.multiply(10, 4)}")
    print(f"Memory: {calc.get_memory()}")

    # Test Fibonacci
    print("\n=== Fibonacci ===")
    fib = rust_py_lib.fibonacci(15)
    print(f"First 15 Fibonacci numbers: {fib}")

    # Test process_numbers
    print("\n=== Process Numbers ===")
    numbers = [1.5, 2.7, 3.2, 4.8, 5.1]
    sum_val, min_val, max_val = rust_py_lib.process_numbers(numbers)
    print(f"Numbers: {numbers}")
    print(f"Sum: {sum_val}, Min: {min_val}, Max: {max_val}")

    # Test string processing
    print("\n=== String Processing ===")
    text = "Hello, World!"
    reversed_text = rust_py_lib.reverse_string(text)
    print(f"Original: {text}")
    print(f"Reversed: {reversed_text}")

    # Test word frequency
    print("\n=== Word Frequency ===")
    text = "the quick brown fox jumps over the lazy dog the fox"
    freq = rust_py_lib.word_frequency(text)
    print(f"Text: {text}")
    print(f"Frequency: {dict(sorted(freq.items(), key=lambda x: x[1], reverse=True))}")

    # Test DataProcessor
    print("\n=== Data Processor ===")
    data = [10, 12, 15, 17, 18, 20, 22, 100]  # 100 is an outlier
    processor = rust_py_lib.DataProcessor(data)
    print(f"Data: {data}")
    print(f"Mean: {processor.mean():.2f}")
    print(f"Median: {processor.median():.2f}")
    print(f"Std Dev: {processor.std_dev():.2f}")
    print(f"Filtered (2σ): {processor.filter_outliers(2.0)}")

if __name__ == "__main__":
    main()
