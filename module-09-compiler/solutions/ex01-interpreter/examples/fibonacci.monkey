// Fibonacci example
let fib = fn(n) {
    if (n <= 1) {
        return n;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
};

// Calculate and print Fibonacci numbers
let i = 0;
while (i < 15) {
    print(fib(i));
    i = i + 1;
}
