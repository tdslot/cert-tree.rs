# Pragmatic Rust Guidelines

## Checklist

- **Universal**
  - ☐ Follow the Upstream Guidelines (M-UPSTREAM-GUIDELINES)
  - ☐ Use Static Verification (M-STATIC-VERIFICATION)
  - ☐ Lint Overrides Should Use `#[expect]` (M-LINT-OVERRIDE-EXPECT)
  - ☐ Public Types are Debug (M-PUBLIC-DEBUG)
  - ☐ Public Types Meant to be Read are Display (M-PUBLIC-DISPLAY)
  - ☐ If in Doubt, Split the Crate (M-SMALLER-CRATES)
  - ☐ Names are Free of Weasel Words (M-CONCISE-NAMES)
  - ☐ Prefer Regular over Associated Functions (M-REGULAR-FN)
  - ☐ Panic Means 'Stop the Program' (M-PANIC-IS-STOP)
  - ☐ Detected Programming Bugs are Panics, Not Errors (M-PANIC-ON-BUG)
  - ☐ All Magic Values and Behaviors are Documented (M-DOCUMENTED-MAGIC)
- **Library / Interoperability**
  - ☐ Types are Send (M-TYPES-SEND)
  - ☐ Native Escape Hatches (M-ESCAPE-HATCHES)
  - ☐ Don't Leak External Types (M-DONT-LEAK-TYPES)
- **Library / UX**
  - ☐ Abstractions Don't Visibly Nest (M-SIMPLE-ABSTRACTIONS)
  - ☐ Avoid Smart Pointers and Wrappers in APIs (M-AVOID-WRAPPERS)
  - ☐ Prefer Types over Generics, Generics over Dyn Traits (M-DI-HIERARCHY)
  - ☐ Error are Canonical Structs (M-ERRORS-CANONICAL-STRUCTS)
  - ☐ Complex Type Construction has Builders (M-INIT-BUILDER)
  - ☐ Complex Type Initialization Hierarchies are Cascaded (M-INIT-CASCADED)
  - ☐ Services are Clone (M-SERVICES-CLONE)
  - ☐ Accept `impl AsRef<>` Where Feasible (M-IMPL-ASREF)
  - ☐ Accept `impl RangeBounds<>` Where Feasible (M-IMPL-RANGEBOUNDS)
  - ☐ Accept `impl 'IO'` Where Feasible ('Sans IO') (M-IMPL-IO)
  - ☐ Essential Functionality Should be Inherent (M-ESSENTIAL-FN-INHERENT)
- **Library / Resilience**
  - ☐ I/O and System Calls Are Mockable (M-MOCKABLE-SYSCALLS)
  - ☐ Test Utilities are Feature Gated (M-TEST-UTIL)
  - ☐ Use the Proper Type Family (M-STRONG-TYPES)
  - ☐ Don't Glob Re-Export Items (M-NO-GLOB-REEXPORTS)
  - ☐ Avoid Statics (M-AVOID-STATICS)
- **Library / Building**
  - ☐ Libraries Work Out of the Box (M-OOBE)
  - ☐ Native `-sys` Crates Compile Without Dependencies (M-SYS-CRATES)
  - ☐ Features are Additive (M-FEATURES-ADDITIVE)
- **Applications**
  - ☐ Use Mimalloc for Apps (M-MIMALLOC-APPS)
  - ☐ Applications may use Anyhow or Derivatives (M-APP-ERROR)
- **FFI**
  - ☐ Isolate DLL State Between FFI Libraries (M-ISOLATE-DLL-STATE)
- **Safety**
  - ☐ Unsafe Needs Reason, Should be Avoided (M-UNSAFE)
  - ☐ Unsafe Implies Undefined Behavior (M-UNSAFE-IMPLIES-UB)
  - ☐ All Code Must be Sound (M-UNSOUND)
- **Performance**
  - ☐ Optimize for Throughput, Avoid Empty Cycles (M-THROUGHPUT)
  - ☐ Identify, Profile, Optimize the Hot Path Early (M-HOTPATH)
  - ☐ Long-Running Tasks Should Have Yield Points (M-YIELD-POINTS)
- **Documentation**
  - ☐ First Sentence is One Line; Approx. 15 Words (M-FIRST-DOC-SENTENCE)
  - ☐ Has Module Documentation (M-MODULE-DOCS)
  - ☐ Documentation Has Canonical Sections (M-CANONICAL-DOCS)
  - ☐ Mark `pub use` Items with `#[doc(inline)]` (M-DOC-INLINE)
- **AI**
  - ☐ Design with AI use in Mind (M-DESIGN-FOR-AI)

## Universal Guidelines

### Follow the Upstream Guidelines (M-UPSTREAM-GUIDELINES)
Adhere to Rust API Guidelines, Rust Style Guide, Rust Design Patterns, and Rust Reference - Undefined Behavior. Pay special attention to C-CONV, C-GETTER, C-COMMON-TRAITS, C-CTOR, C-FEATURE.

### Use Static Verification (M-STATIC-VERIFICATION)
Use compiler lints, clippy, rustfmt, cargo-audit, cargo-hack, cargo-udeps, miri. Enable specific lints in Cargo.toml.

### Lint Overrides Should Use `#[expect]` (M-LINT-OVERRIDE-EXPECT)
Use `#[expect(clippy::lint_name, reason = "...")]` instead of `#[allow]` to prevent accumulation of outdated lints.

### Public Types are Debug (M-PUBLIC-DEBUG)
All public types should implement `Debug`. Use `#[derive(Debug)]` or custom impl for sensitive data with tests.

### Public Types Meant to be Read are Display (M-PUBLIC-DISPLAY)
Types meant to be read by users should implement `Display`, including error types and string-like wrappers.

### If in Doubt, Split the Crate (M-SMALLER-CRATES)
Err on the side of more crates rather than fewer to improve compile times and modularity. Split if a submodule can be used independently.

### Names are Free of Weasel Words (M-CONCISE-NAMES)
Avoid weasel words like Service, Manager, Factory. Use specific names. Prefer Builder over Factory.

### Prefer Regular over Associated Functions (M-REGULAR-FN)
Use regular functions over associated functions for non-instance logic. Associated functions primarily for instance creation.

### Panic Means 'Stop the Program' (M-PANIC-IS-STOP)
Panics terminate the program. Don't use for recoverable errors. Valid for programming errors, const contexts, user requests, poison.

### Detected Programming Bugs are Panics, Not Errors (M-PANIC-ON-BUG)
Panic on detected programming errors. Contract violations are programming errors.

### All Magic Values and Behaviors are Documented (M-DOCUMENTED-MAGIC)
Hardcoded values must be documented with why chosen, side effects, external systems. Prefer named constants.

## Library Guidelines / Interoperability

### Types are Send (M-TYPES-SEND)
Public types should be `Send` for Tokio compatibility. Futures must be `Send`. Exceptions for instantaneous use.

### Native Escape Hatches (M-ESCAPE-HATCHES)
Provide `unsafe` conversion methods for native handles in interop scenarios.

### Don't Leak External Types (M-DONT-LEAK-TYPES)
Prefer `std` types in public APIs. Exceptions for substantial benefits or feature-gated.

## Library Guidelines / UX

### Abstractions Don't Visibly Nest (M-SIMPLE-ABSTRACTIONS)
Avoid nested parametrized types in public APIs. Use simple types, limit nesting to 1 level.

### Avoid Smart Pointers and Wrappers in APIs (M-AVOID-WRAPPERS)
Avoid `Rc<T>`, `Arc<T>`, `Box<T>`, `RefCell<T>` in public APIs. Hide internally.

### Prefer Types over Generics, Generics over Dyn Traits (M-DI-HIERARCHY)
For async dependencies: concrete types > generics > dyn Trait. Use enum for testing.

### Error are Canonical Structs (M-ERRORS-CANONICAL-STRUCTS)
Errors as structs with Backtrace, cause, helper methods. Implement Display, std::error::Error.

### Complex Type Construction has Builders (M-INIT-BUILDER)
Types with 4+ parameters should have builders. Builder methods chainable, final `.build()`.

### Complex Type Initialization Hierarchies are Cascaded (M-INIT-CASCADED)
Group parameters semantically for 4+ parameters.

### Services are Clone (M-SERVICES-CLONE)
Heavyweight service types implement Clone semantics, using Arc<Inner>.

### Accept `impl AsRef<>` Where Feasible (M-IMPL-ASREF)
Accept `impl AsRef<T>` for flexible input where ownership not needed.

### Accept `impl RangeBounds<>` Where Feasible (M-IMPL-RANGEBOUNDS)
Use `impl RangeBounds<T>` over hand-rolled parameters.

### Accept `impl 'IO'` Where Feasible ('Sans IO') (M-IMPL-IO)
Use sans-io pattern: accept `impl Read`/`impl Write` for one-shot I/O.

### Essential Functionality Should be Inherent (M-ESSENTIAL-FN-INHERENT)
Implement core functionality inherently, forward to traits.

## Library Guidelines / Resilience

### I/O and System Calls Are Mockable (M-MOCKABLE-SYSCALLS)
Make I/O and sys calls mockable. Accept cores or provide mocking via feature.

### Test Utilities are Feature Gated (M-TEST-UTIL)
Guard testing functionality behind feature flag, e.g., `test-util`.

### Use the Proper Type Family (M-STRONG-TYPES)
Use strongest `std` type available early. Prefer `PathBuf` over `String` for paths.

### Don't Glob Re-Export Items (M-NO-GLOB-REEXPORTS)
Re-export individually, not `pub use foo::*`.

### Avoid Statics (M-AVOID-STATICS)
Avoid `static` where consistency matters. Can cause issues with multiple crate versions.

## Library Guidelines / Building

### Libraries Work Out of the Box (M-OOBE)
Build on all Tier 1 platforms without prerequisites beyond cargo/rust.

### Native `-sys` Crates Compile Without Dependencies (M-SYS-CRATES)
Govern native lib build from build.rs using cc crate.

### Features are Additive (M-FEATURES-ADDITIVE)
Features must be additive, no disabling public items.

## Application Guidelines

### Use Mimalloc for Apps (M-MIMALLOC-APPS)
Set mimalloc as global allocator for performance gains.

### Applications may use Anyhow or Derivatives (M-APP-ERROR)
Apps may use anyhow/eyre for error handling.

## FFI Guidelines

### Isolate DLL State Between FFI Libraries (M-ISOLATE-DLL-STATE)
Share only portable state between DLLs. Avoid `static`, `TypeId`, allocated types.

## Safety Guidelines

### Unsafe Needs Reason, Should be Avoided (M-UNSAFE)
Use `unsafe` only for novel abstractions, performance, FFI. Must have reasoning, pass Miri.

### Unsafe Implies Undefined Behavior (M-UNSAFE-IMPLIES-UB)
`unsafe` only for UB risk, not other dangers.

### All Code Must be Sound (M-UNSOUND)
No unsound code. Safe functions must not cause UB.

## Performance Guidelines

### Optimize for Throughput, Avoid Empty Cycles (M-THROUGHPUT)
Optimize for items/CPU cycle. Batch work, avoid contended locks.

### Identify, Profile, Optimize the Hot Path Early (M-HOTPATH)
Identify hot paths early, benchmark, profile.

### Long-Running Tasks Should Have Yield Points (M-YIELD-POINTS)
Add `yield_now().await` in long-running tasks.

## Documentation

### First Sentence is One Line; Approx. 15 Words (M-FIRST-DOC-SENTENCE)
First sentence <15 words for skimmable docs.

### Has Module Documentation (M-MODULE-DOCS)
Public modules need `//!` docs with comprehensive info.

### Documentation Has Canonical Sections (M-CANONICAL-DOCS)
Use # Examples, # Errors, # Panics, # Safety sections.

### Mark `pub use` Items with `#[doc(inline)]` (M-DOC-INLINE)
Use `#[doc(inline)]` for re-exports.

## AI Guidelines

### Design with AI use in Mind (M-DESIGN-FOR-AI)
Follow idiomatic patterns, provide docs, examples, strong types, testable APIs, test coverage.
