// Example usage of Rust WASM module in TypeScript
import init, {
  greet,
  fibonacci,
  sum_array,
  analyze_text,
  DataProcessor,
  process_user,
  hash_string,
  is_prime,
  primes_up_to,
} from '../pkg/rust_wasm_lib';

async function main() {
  // Initialize the WASM module
  await init();

  console.log('=== WASM Module Demo ===\n');

  // Simple greeting
  console.log('1. Greeting:');
  const greeting = greet('TypeScript');
  console.log(greeting);
  console.log();

  // Fibonacci calculation
  console.log('2. Fibonacci Numbers:');
  for (let i of [5, 10, 20, 30]) {
    console.log(`fibonacci(${i}) = ${fibonacci(i)}`);
  }
  console.log();

  // Array processing
  console.log('3. Array Processing:');
  const numbers = new Float64Array([1.5, 2.5, 3.5, 4.5, 5.5]);
  const sum = sum_array(numbers);
  console.log(`Sum of [${Array.from(numbers).join(', ')}] = ${sum}`);
  console.log();

  // Text analysis
  console.log('4. Text Analysis:');
  const text = `The quick brown fox jumps over the lazy dog.
The dog was really lazy.
The fox was very quick and brown.`;
  const stats = analyze_text(text);
  console.log('Text:', text);
  console.log('Stats:', stats);
  console.log();

  // DataProcessor class
  console.log('5. DataProcessor Class:');
  const processor = new DataProcessor();
  processor.add_data([10, 12, 15, 17, 18, 20, 22, 100]);
  console.log(`Length: ${processor.len()}`);
  console.log(`Mean: ${processor.mean().toFixed(2)}`);
  console.log(`Median: ${processor.median().toFixed(2)}`);
  console.log(`Std Dev: ${processor.std_dev().toFixed(2)}`);
  console.log();

  // Process user data
  console.log('6. User Processing:');
  const userData = {
    name: 'Alice',
    email: 'alice@example.com',
    age: 30,
  };
  const userResult = process_user(userData);
  console.log('User:', userData);
  console.log('Result:', userResult);
  console.log();

  // String hashing
  console.log('7. String Hashing:');
  const strings = ['hello', 'world', 'rust', 'wasm', 'typescript'];
  for (const str of strings) {
    console.log(`hash("${str}") = ${hash_string(str)}`);
  }
  console.log();

  // Prime number checking
  console.log('8. Prime Number Checking:');
  const testNumbers = [2, 3, 4, 5, 17, 20, 23, 100, 101];
  for (const num of testNumbers) {
    console.log(`is_prime(${num}) = ${is_prime(num)}`);
  }
  console.log();

  // Find primes up to N
  console.log('9. Primes up to 50:');
  const primes = primes_up_to(50);
  console.log(primes);
  console.log();

  // Performance comparison: Fibonacci
  console.log('10. Performance Test (Fibonacci):');

  // JavaScript implementation
  function fibJS(n: number): number {
    if (n <= 1) return n;
    let prev = 0, curr = 1;
    for (let i = 2; i <= n; i++) {
      const next = prev + curr;
      prev = curr;
      curr = next;
    }
    return curr;
  }

  const n = 40;
  const iterations = 100000;

  // WASM benchmark
  const wasmStart = performance.now();
  for (let i = 0; i < iterations; i++) {
    fibonacci(n);
  }
  const wasmTime = performance.now() - wasmStart;

  // JavaScript benchmark
  const jsStart = performance.now();
  for (let i = 0; i < iterations; i++) {
    fibJS(n);
  }
  const jsTime = performance.now() - jsStart;

  console.log(`Computing fibonacci(${n}) ${iterations} times:`);
  console.log(`WASM: ${wasmTime.toFixed(2)}ms`);
  console.log(`JavaScript: ${jsTime.toFixed(2)}ms`);
  console.log(`Speedup: ${(jsTime / wasmTime).toFixed(2)}x`);
  console.log();

  console.log('=== Demo Complete ===');
}

// Run the demo
main().catch(console.error);
