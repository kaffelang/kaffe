# Kaffe

**Kaffe** is an experimental readable, type-safe programming language somewhere between CoffeeScript and TypeScript.

The goal is simple:

> CoffeeScript readability, TypeScript confidence, fast tooling.

Kaffe is not trying to be “JavaScript with slightly nicer syntax”. It is a small typed source language designed to compile to boring, readable TypeScript or JavaScript.

## Status

Kaffe is currently in the earliest design/prototype phase.

The first milestone is to build a small vertical slice:

```text
.kaf source
  -> lexer
  -> parser
  -> AST
  -> TypeScript emitter
  -> fast CLI build
```

## Why Kaffe?

Modern TypeScript is powerful, but it can become visually noisy and complex. CoffeeScript was pleasant to read, but it did not solve type safety.

Kaffe tries to sit in the useful middle:

- readable indentation-based syntax
- static types
- simple function declarations
- object and type literals that are easy to scan
- expression-oriented control flow
- minimal runtime
- fast parser and builder
- TypeScript/JavaScript output that humans can still read

## Example

```kaffe
type User =
  id: string
  name: string
  age?: number
  role: "admin" | "editor" | "reader"

+fn canEdit(user: User): boolean =>
  user.role in ["admin", "editor"]

+fn greet(user: User): string =>
  "Hello, #{user.name}"
```

Expected TypeScript output:

```ts
type User = {
  id: string
  name: string
  age?: number
  role: "admin" | "editor" | "reader"
}

export function canEdit(user: User): boolean {
  return ["admin", "editor"].includes(user.role)
}

export function greet(user: User): string {
  return `Hello, ${user.name}`
}
```

## Design principles

### Readability first

Kaffe code should be easy to scan and easy to explain.

```kaffe
fn fullName(user: User): string =>
  "#{user.firstName} #{user.lastName}"
```

### Type safety without ceremony

Kaffe should catch common mistakes without turning everyday code into type gymnastics.

```kaffe
type Product =
  id: string
  name: string
  price: number
  tags: string[]
```

### Fast as a core feature

The parser, checker, emitter, and builder should be designed for speed from the beginning.

Target direction:

```text
fast cold builds
near-instant warm rebuilds
low memory usage
incremental parsing/checking/emitting
single fast CLI binary
```

### Minimal runtime

The best runtime is no runtime.

Kaffe should compile to simple TypeScript/JavaScript and only use helpers when they provide clear value.

### Boring output

Generated TypeScript/JavaScript should be readable enough to inspect, debug, and trust.

## File extension

Preferred source extension:

```text
.kaf
```

Example:

```text
src/main.kaf
```

## Planned CLI

```bash
kaffe parse src/main.kaf
kaffe build src/main.kaf --out dist/main.ts
kaffe check
kaffe dev
kaffe fmt
```

Initial focus:

```bash
kaffe parse
kaffe build
```

## Initial syntax goals

### Type aliases

```kaffe
type User =
  id: string
  name: string
  age?: number
```

### Function declarations

```kaffe
fn add(a: number, b: number): number =>
  a + b
```

### Exported functions

```kaffe
+fn hello(name: string): string =>
  "Hello, #{name}"
```

### If expressions

```kaffe
fn label(user: User): string =>
  if user.role == "admin"
    "Admin"
  else
    "User"
```

### Arrays

```kaffe
roles = ["admin", "editor", "reader"]
```

### `in` operator

```kaffe
user.role in ["admin", "editor"]
```

### Template strings

```kaffe
"Hello, #{user.name}"
```

## Non-goals for the first prototype

The first version should not try to solve everything.

Out of scope for the first vertical slice:

- full type checker
- language server
- formatter
- package manager
- JSX
- classes
- pattern matching
- async/await
- macros
- decorators
- source maps
- custom runtime

These may come later, but the first goal is proving the compiler pipeline.

## Suggested architecture

The toolchain should be small, modular, and fast.

```text
kaffe/
  crates/
    kaffe_ast/
    kaffe_parser/
    kaffe_emit_ts/
    kaffe_cli/
  examples/
  tests/
  README.md
```

Possible pipeline:

```text
source text
  -> lexer
  -> parser
  -> AST
  -> TypeScript emitter
```

Later:

```text
source text
  -> lexer
  -> parser
  -> AST
  -> resolver
  -> type checker
  -> lowered IR
  -> JavaScript/TypeScript emitter
```

## Implementation direction

Rust is a good fit for the Kaffe toolchain because it gives us:

- fast CLI startup
- predictable performance
- memory control
- good error handling
- safe concurrency
- single-binary distribution

A hand-written lexer/parser is preferred for the first version. The language should stay small enough that the parser remains understandable.

## Roadmap

### Milestone 1: Lexer and parser

- [ ] Tokenize `.kaf` source
- [ ] Support indentation-aware tokens
- [ ] Parse type aliases
- [ ] Parse function declarations
- [ ] Parse basic expressions
- [ ] Produce a simple AST
- [ ] Print AST as debug output

### Milestone 2: TypeScript emitter

- [ ] Emit type aliases
- [ ] Emit functions
- [ ] Emit string, number, boolean, array, and object literals
- [ ] Emit template strings
- [ ] Emit `if` expressions
- [ ] Emit `in` as `.includes(...)`
- [ ] Preserve readable output

### Milestone 3: CLI

- [ ] `kaffe parse`
- [ ] `kaffe build`
- [ ] Useful syntax errors with line/column
- [ ] Basic test suite

### Milestone 4: Basic checker

- [ ] Unknown variables
- [ ] Wrong function argument count
- [ ] Invalid primitive assignments
- [ ] Missing required object fields
- [ ] Invalid literal union values
- [ ] Return type mismatch

### Milestone 5: Fast builder

- [ ] File graph
- [ ] Content hashing
- [ ] Incremental rebuilds
- [ ] Watch mode
- [ ] Parallel emit

## Example project

```text
example/
  src/
    main.kaf
  kaffe.toml
```

Possible `kaffe.toml`:

```toml
[project]
src = "src"
out = "dist"
target = "ts"

[build]
sourceMaps = false
declaration = false
```

## Philosophy

Kaffe should prefer this:

```kaffe
type Person =
  name: string
  age: number

fn describe(person: Person): string =>
  if person.age < 18
    "#{person.name} is a child"
  else
    "#{person.name} is an adult"
```

Over this:

```ts
type Person = {
  name: string;
  age: number;
};

function describe(person: Person): string {
  if (person.age < 18) {
    return `${person.name} is a child`;
  }

  return `${person.name} is an adult`;
}
```

The goal is not to remove all punctuation. The goal is to remove visual noise while keeping precision.

## Inspiration

Kaffe is inspired by:

- CoffeeScript readability
- TypeScript type safety
- Elm-style correctness
- Rust-style tooling discipline
- Go-style boring output and fast tools

## Contributing

The project is early. The most useful contributions right now are:

- syntax design experiments
- parser tests
- AST design discussions
- TypeScript emitter work
- benchmarks
- examples of real-world code rewritten in Kaffe

Before adding large features, prefer small vertical slices that can be parsed, emitted, tested, and benchmarked.

## License

License not decided yet.

A permissive open-source license such as MIT or Apache-2.0 is likely a good fit.
