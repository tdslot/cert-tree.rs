# Rust Performance Guidelines: Blazing Fast Code

This document outlines comprehensive coding guidelines for writing high-performance Rust code that embodies the "Blazing Fast" philosophy. These rules prioritize zero-cost abstractions, efficient memory management, safe concurrency, and minimal runtime overhead while maintaining Rust's core principles of safety, speed, and concurrency.

## 1. Zero-Cost Abstractions

**Rule 1.1: Leverage compile-time optimizations over runtime checks**
- Use const generics and compile-time evaluation for performance-critical code
- Prefer static dispatch over dynamic dispatch when polymorphism isn't needed
- Utilize const fn for operations that can be evaluated at compile time

**Example:**
```rust
// Good: Compile-time size checking
fn process_array<const N: usize>(arr: &[i32; N]) -> i32 {
    arr.iter().sum()
}

// Avoid: Runtime size checking
fn process_vec(vec: &Vec<i32>) -> i32 {
    if vec.len() > 1000 { panic!("Too large"); }
    vec.iter().sum()
}
```

**Justification:** Compile-time checks eliminate runtime overhead while maintaining safety guarantees.

**Pitfall:** Overusing const generics can increase compile times; use judiciously for hot paths.

**Rule 1.2: Prefer stack allocation for small, fixed-size data**
- Use arrays and tuples for small data structures
- Avoid heap allocation for temporary or small objects

**Example:**
```rust
// Good: Stack-allocated matrix
fn matrix_multiply<const N: usize>(a: &[[f32; N]; N], b: &[[f32; N]; N]) -> [[f32; N]; N] {
    let mut result = [[0.0; N]; N];
    // Implementation
    result
}
```

**Justification:** Stack allocation is faster and doesn't require garbage collection or manual memory management.

## 2. Memory Management

**Rule 2.1: Minimize allocations in hot loops**
- Pre-allocate containers with known capacity
- Reuse buffers and avoid creating temporary objects

**Example:**
```rust
// Good: Pre-allocated vector
fn process_data(data: &[u8]) -> Vec<String> {
    let mut result = Vec::with_capacity(data.len() / 10); // Estimate capacity
    for chunk in data.chunks(10) {
        result.push(String::from_utf8_lossy(chunk).to_string());
    }
    result
}

// Avoid: Repeated allocations
fn process_data_slow(data: &[u8]) -> Vec<String> {
    let mut result = Vec::new(); // No capacity hint
    for chunk in data.chunks(10) {
        result.push(String::from_utf8_lossy(chunk).to_string()); // Allocates each time
    }
    result
}
```

**Justification:** Allocations are expensive; minimizing them reduces GC pressure and improves cache locality.

**Pitfall:** Over-estimating capacity wastes memory; profile to find optimal values.

**Rule 2.2: Use borrowing to avoid unnecessary copies**
- Prefer &T over T when ownership transfer isn't needed
- Use slices for partial data access

**Example:**
```rust
// Good: Borrowed data
fn find_max(slice: &[i32]) -> Option<&i32> {
    slice.iter().max()
}

// Avoid: Owned copies
fn find_max_owned(vec: Vec<i32>) -> Option<i32> {
    vec.into_iter().max()
}
```

**Justification:** Borrowing eliminates copy overhead while maintaining safety through lifetimes.

**Rule 2.3: Optimize data layout for cache efficiency**
- Group frequently accessed fields together in structs
- Use smaller types when possible (u32 vs u64, f32 vs f64)

**Example:**
```rust
// Good: Optimized struct layout
#[repr(C)] // Ensures C-compatible layout
struct Particle {
    x: f32, y: f32, z: f32, // Position first (hot data)
    vx: f32, vy: f32, vz: f32, // Velocity next
    mass: f32, // Less frequently accessed
    id: u32,
}
```

**Justification:** Better cache locality reduces memory access latency.

## 3. Data Structures

**Rule 3.1: Choose the right collection for the access pattern**
- Vec<T> for sequential access and growth
- HashMap<K, V> for O(1) lookups (consider hash quality)
- BTreeMap<K, V> for ordered data and range queries
- HashSet<T> for unique membership testing

**Example:**
```rust
// Good: Vec for iteration
let mut primes = Vec::with_capacity(1000);
for i in 2.. {
    if is_prime(i) { primes.push(i); }
    if primes.len() == 1000 { break; }
}

// Good: HashSet for fast lookups
let mut seen = HashSet::with_capacity(input.len());
for item in input {
    if !seen.contains(item) {
        seen.insert(item);
        process(item);
    }
}
```

**Justification:** Each data structure has different performance characteristics; choose based on usage patterns.

**Pitfall:** HashMap with poor hash functions can degrade to O(n) lookups.

**Rule 3.2: Use small vector optimizations**
- Consider SmallVec or ArrayVec for small, bounded collections
- Use enum variants for optional fields to avoid Option<T> overhead

**Example:**
```rust
use smallvec::SmallVec;

// Good: Small vector for small collections
type SmallStringVec = SmallVec<[String; 4]>;

fn collect_strings(iter: impl Iterator<Item = String>) -> SmallStringVec {
    iter.collect()
}
```

**Justification:** Avoids heap allocation for small collections, improving performance for common cases.

## 4. Algorithms and Iteration

**Rule 4.1: Prefer iterators over manual loops**
- Use iterator adapters for transformation and filtering
- Leverage collect() with type hints for optimal collection

**Example:**
```rust
// Good: Iterator chains
let result: Vec<i32> = data.iter()
    .filter(|&&x| x > 0)
    .map(|&x| x * 2)
    .take(10)
    .collect();

// Avoid: Manual loops
let mut result = Vec::new();
for &x in &data {
    if x > 0 {
        result.push(x * 2);
        if result.len() == 10 { break; }
    }
}
```

**Justification:** Iterators enable compiler optimizations and are often more readable and efficient.

**Rule 4.2: Use rayon for data-parallel computations**
- Parallelize CPU-bound tasks with Rayon
- Use par_iter() for embarrassingly parallel operations

**Example:**
```rust
use rayon::prelude::*;

// Good: Parallel processing
let sum: i64 = large_vec.par_iter().map(|&x| expensive_computation(x)).sum();
```

**Justification:** Leverages multiple cores for better performance on large datasets.

**Pitfall:** Parallelization overhead can hurt performance for small datasets; measure before applying.

## 5. Error Handling

**Rule 5.1: Use Result for recoverable errors, avoid panics in libraries**
- Prefer Result<T, E> over Option<T> when error context is needed
- Use thiserror for ergonomic error types

**Example:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    Parse(String),
}

// Good: Explicit error handling
fn read_file(path: &Path) -> Result<String, MyError> {
    let content = std::fs::read_to_string(path)?;
    Ok(content)
}
```

**Justification:** Explicit error handling avoids unexpected panics while maintaining performance.

**Rule 5.2: Avoid unnecessary error allocations**
- Use static strings for error messages when possible
- Implement Display manually for custom error types to avoid formatting overhead

**Example:**
```rust
// Good: Static error messages
#[derive(Debug)]
pub struct ParseError(&'static str);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
```

**Justification:** Reduces allocations in error paths, which are often cold but still impact performance.

## 6. Concurrency

**Rule 6.1: Use channels for message passing**
- Prefer crossbeam or std::sync::mpsc for producer-consumer patterns
- Use bounded channels to prevent unbounded memory growth

**Example:**
```rust
use crossbeam::channel::{bounded, Receiver, Sender};

fn producer(tx: Sender<i32>) {
    for i in 0..100 {
        tx.send(i).unwrap();
    }
}

fn consumer(rx: Receiver<i32>) {
    while let Ok(data) = rx.recv() {
        process(data);
    }
}

// Good: Bounded channel
let (tx, rx) = bounded(10);
```

**Justification:** Channels provide safe communication between threads without shared mutable state.

**Rule 6.2: Minimize lock contention**
- Hold locks for minimal time
- Use RwLock for read-heavy workloads
- Consider lock-free data structures for high-contention scenarios

**Example:**
```rust
use std::sync::RwLock;

// Good: Read-write lock for read-heavy access
let data = RwLock::new(vec![1, 2, 3]);

// Reader
let guard = data.read().unwrap();
let sum: i32 = guard.iter().sum();

// Writer (exclusive access)
*data.write().unwrap() = vec![4, 5, 6];
```

**Justification:** Reduces thread contention and improves scalability.

**Pitfall:** Overusing locks can lead to deadlocks; prefer message passing when possible.

## 7. Benchmarking and Profiling

**Rule 7.1: Use Criterion for microbenchmarks**
- Benchmark hot functions with realistic data
- Compare implementations quantitatively

**Example:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 1,
        1 => 1,
        n => fibonacci(n-1) + fibonacci(n-2),
    }
}

fn bench_fib(c: &mut Criterion) {
    c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
}

criterion_group!(benches, bench_fib);
criterion_main!(benches);
```

**Justification:** Criterion provides statistically sound benchmarking with proper warmup and outlier detection.

**Rule 7.2: Profile with perf or flamegraph**
- Use `cargo flamegraph` for CPU profiling
- Identify bottlenecks before optimizing

**Example:**
```bash
# Profile release build
cargo flamegraph --bin my_app -- --input data.txt
```

**Justification:** Profiling reveals true performance bottlenecks, guiding optimization efforts.

**Rule 7.3: Measure end-to-end performance**
- Benchmark complete workflows, not just functions
- Consider memory usage and allocation patterns

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_performance() {
        let data = generate_large_dataset();
        let start = Instant::now();
        let result = process_data(data);
        let elapsed = start.elapsed();
        assert!(elapsed < std::time::Duration::from_millis(100));
        // Also check result correctness
    }
}
```

**Justification:** End-to-end benchmarks ensure optimizations improve real-world performance.

## 8. Common Pitfalls to Avoid

**Pitfall 1: Unnecessary boxing**
```rust
// Bad: Boxing small data
let data: Box<[u8; 1024]> = Box::new([0; 1024]);

// Good: Stack allocation
let data: [u8; 1024] = [0; 1024];
```

**Pitfall 2: String concatenation in loops**
```rust
// Bad: Repeated allocations
let mut result = String::new();
for word in words {
    result.push_str(word);
    result.push(' ');
}

// Good: Pre-calculate size
let total_len = words.iter().map(|w| w.len() + 1).sum();
let mut result = String::with_capacity(total_len);
for word in words {
    result.push_str(word);
    result.push(' ');
}
```

**Pitfall 3: Ignoring SIMD opportunities**
```rust
// Consider using SIMD for numerical computations
// Libraries like faster, or std::simd (nightly) can help
```

**Pitfall 4: Over-optimization**
- Don't optimize code that's not a bottleneck
- Profile first, then optimize
- Maintain code readability unless performance gains are significant

## Conclusion

Writing blazing fast Rust code requires understanding both Rust's unique features and general performance principles. Focus on measuring performance, minimizing allocations, and leveraging Rust's zero-cost abstractions. Always profile before optimizing, and remember that premature optimization is the root of all evil.

These guidelines should be applied judiciously based on your specific use case and performance requirements. The key is to write correct, maintainable code first, then optimize the bottlenecks identified through profiling.