# Pragmatic Rust Guidelines

This document outlines key pragmatic guidelines for Rust development, adapted from the Microsoft Rust Guidelines (https://microsoft.github.io/rust-guidelines/print.html). These guidelines help produce idiomatic, safe, and maintainable Rust code that scales.

## Meta Design Principles

Guidelines must positively affect safety, COGs (cost of goods sold), or maintenance. They should be agreed upon by experienced Rust developers, comprehensible to novices, and pragmatic.

## Universal Guidelines

### Follow the Upstream Guidelines (M-UPSTREAM-GUIDELINES)
- Adhere to Rust API Guidelines, Rust Style Guide, and Rust Design Patterns.
- Key items: C-CONV, C-GETTER, C-COMMON-TRAITS, C-CTOR, C-FEATURE.

### Use Static Verification (M-STATIC-VERIFICATION)
- Enable compiler lints, Clippy, rustfmt, cargo-audit, cargo-hack, cargo-udeps, Miri.
- Compiler lints: ambiguous_negative_literals, missing_debug_implementations, etc.
- Clippy categories: cargo, complexity, correctness, pedantic, perf, style, suspicious.

### Lint Overrides Should Use `#[expect]` (M-LINT-OVERRIDE-EXPECT)
- Use `#[expect(clippy::lint_name, reason = "...")]` instead of `#[allow]`.

### Public Types are Debug (M-PUBLIC-DEBUG)
- Implement `Debug` for all public types, with custom impl for sensitive data.

### Public Types Meant to be Read are Display (M-PUBLIC-DISPLAY)
- Implement `Display` for readable public types.

### If in Doubt, Split the Crate (M-SMALLER-CRATES)
- Prefer smaller crates over larger ones for compile times and modularity.

### Names are Free of Weasel Words (M-CONCISE-NAMES)
- Avoid words like Service, Manager, Factory; use specific names.

### Prefer Regular over Associated Functions (M-REGULAR-FN)
- Use regular functions over associated functions for non-instance logic.

### Panic Means 'Stop the Program' (M-PANIC-IS-STOP)
- Panics terminate the program; don't use for recoverable errors.

### Detected Programming Bugs are Panics, Not Errors (M-PANIC-ON-BUG)
- Panic on detected bugs, not user errors.

### All Magic Values and Behaviors are Documented (M-DOCUMENTED-MAGIC)
- Document hardcoded values with why, side effects, external systems.

## Library Guidelines

### Interoperability
- Types are Send (M-TYPES-SEND): Public types should be Send.
- Native Escape Hatches (M-ESCAPE-HATCHES): Provide unsafe conversions for native handles.
- Don't Leak External Types (M-DONT-LEAK-TYPES): Prefer std types in public APIs.

### UX
- Abstractions Don't Visibly Nest (M-SIMPLE-ABSTRACTIONS): Avoid complex nested generics.
- Avoid Smart Pointers and Wrappers in APIs (M-AVOID-WRAPPERS): Use simple types.
- Prefer Types over Generics, Generics over Dyn Traits (M-DI-HIERARCHY).
- Error are Canonical Structs (M-ERRORS-CANONICAL-STRUCTS): Use structs with Backtrace.
- Complex Type Construction has Builders (M-INIT-BUILDER).
- Complex Type Initialization Hierarchies are Cascaded (M-INIT-CASCADED).
- Services are Clone (M-SERVICES-CLONE).
- Accept `impl AsRef<>` Where Feasible (M-IMPL-ASREF).
- Accept `impl RangeBounds<>` Where Feasible (M-IMPL-RANGEBOUNDS).
- Accept `impl 'IO'` Where Feasible ('Sans IO') (M-IMPL-IO).
- Essential Functionality Should be Inherent (M-ESSENTIAL-FN-INHERENT).

### Resilience
- I/O and System Calls Are Mockable (M-MOCKABLE-SYSCALLS).
- Test Utilities are Feature Gated (M-TEST-UTIL).
- Use the Proper Type Family (M-STRONG-TYPES).
- Don't Glob Re-Export Items (M-NO-GLOB-REEXPORTS).
- Avoid Statics (M-AVOID-STATICS).

### Building
- Libraries Work Out of the Box (M-OOBE).
- Native `-sys` Crates Compile Without Dependencies (M-SYS-CRATES).
- Features are Additive (M-FEATURES-ADDITIVE).

## Application Guidelines

- Use Mimalloc for Apps (M-MIMALLOC-APPS): Set mimalloc as global allocator.
- Applications may use Anyhow or Derivatives (M-APP-ERROR).

## FFI Guidelines

- Isolate DLL State Between FFI Libraries (M-ISOLATE-DLL-STATE).

## Safety Guidelines

- Unsafe Needs Reason, Should be Avoided (M-UNSAFE).
- All Code Must be Sound (M-UNSOUND).
- Unsafe Implies Undefined Behavior (M-UNSAFE-IMPLIES-UB).

## Performance Guidelines

- Optimize for Throughput, Avoid Empty Cycles (M-THROUGHPUT).
- Identify, Profile, Optimize the Hot Path Early (M-HOTPATH).
- Long-Running Tasks Should Have Yield Points (M-YIELD-POINTS).

## Documentation Guidelines

- First Sentence is One Line; Approx. 15 Words (M-FIRST-DOC-SENTENCE).
- Has Module Documentation (M-MODULE-DOCS).
- Documentation Has Canonical Sections (M-CANONICAL-DOCS).
- Mark `pub use` Items with `#[doc(inline)]` (M-DOC-INLINE).

## AI Guidelines

- Design with AI use in Mind (M-DESIGN-FOR-AI): Follow idiomatic patterns, provide docs and examples.

## Checklist

See the full checklist at https://microsoft.github.io/rust-guidelines/print.html#checklist for all items.

## Contributing

File issues or PRs at https://github.com/microsoft/rust-guidelines.