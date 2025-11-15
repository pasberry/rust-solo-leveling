// Closure example
let newAdder = fn(x) {
    fn(y) { x + y }
};

let add5 = newAdder(5);
let add10 = newAdder(10);

print(add5(3));   // 8
print(add10(3));  // 13

// Higher-order functions
let apply = fn(f, x) { f(x) };
let double = fn(x) { x * 2 };

print(apply(double, 21));  // 42
