<p align="center">
<img src="./additional/icon/banner.png">
    <i>The most damaging phrase in the language is.. it's always been done this way</i>
</p>

<p align="center">
<details>
  <summary>We're on Discord and Telegram!</summary>
  We have a <a href="https://discord.gg/re29xvSV2">Discord server</a> and a <a href="https://t.me/ry_lang">Telegram group</a>.
</details>
</p>

## Table of contents

- [Introduction](#Introduction)
- [Installation](#Installation)
- [Overview](#Overview)

# Introduction

Ry is an open source programming language for web development with expressive type system and easy-to-learn syntax that makes it easy to build reliable and efficient software.

Example of hello world program:

```
pub fun main() {
    println("hello world");
}
```

# Installation

## Compiling from source code

You need to have Rust installed on your system. Then run:

```
cargo install --path crates/ry
```

# Overview

# Pattern matching

Ry supports matching patterns by having a `match` expression:

```
match tuple {
    (1, ..) => {
        println("First element is 1");
    }
    (.., 'b', true) | (.., 'a', true) => {
        println("Second element is 'b' or 'a', and third element is true");
    }
    (.., false) => {
        println("Third element is false");
    }
    .. => {
        println("Default case");
    }
}
```

Pattern matching can also be used in `let` statement for destructuring:

```
let Person {
    name,
    age,
} = get_person();
```

# Everything is expression

Ry follows "everything is expression" philosophy. So `if`, `match`, `while`, etc. are expressions:

```
fun factorial(n: uint32): uint32 {
    if n < 2 {
        1
    } else {
        factorial(n - 1) * n
    }
}
```

# Type system

Ry also supports Rust trait system:

```
trait Foo {
    fun foo();
}

impl Foo for Bar {
    fun foo() {
        println("foo");
    }
}
```

With generics, associated types and type aliases:

```
trait Iterator {
    type Item;

    fun next(self): Option[Self.Item];
}

trait Add[RHS = Self] {
    type Output;

    fun add(self, rhs: RHS): Self.Output;
}

type HashMapItem[K, V] = [HashMap[K, V] as IntoIterator].Item;
```

Ry also supports super traits:

```
trait MyNumeric: Numeric {}

impl MyNumeric for Complex { ... }
```

and negative trait bounds:

```
fun not_default[T](n: T) where T: Not[Default] {
    ...
}

fun main() {
    not_default(3); // error (numbers implement Default trait)
}
```

Ry supports function types:

```
fun do_stuff_with(a: uint32, b: uint32, fn: (uint32, uint32): ()) {
    fn(a, b)
}
```

The language also has an analog of sum types: _enums_:

```
enum Result[T, E] {
    Ok(T),
    Err(E),
}
```

and error propagation:

```
fun safe_div[T](a: T, b: T): Option[T] where T: Numeric {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

fun main() {
    let a = safe_div(1, 1)?;
    assert(a == 1);

    safe_div(1, 0)?;
}
```

If type implements 2 traits having functions with the same names, you can use type qualification:

```
struct S {}

impl S {
    fun f() { println("S"); }
}

trait T1 {
    fun f() { println("T1 f"); }
}

impl T1 for S {}

trait T2 {
    fun f() { println("T2 f"); }
}

impl T2 for S {}

fun main() {
    S.f(); // S
    [S as T1].f(); // T1 f
    [S as T2].f(); // T2 f
}
```

If you want to have to deal with dynamic dispatch, you can use `dyn` type:

```
fun main() {
    let iter = [1, 2, 3].into_iter() as dyn Iterator[Item = uint32];

    assert(iter.next() == Some(1));
}
```

Ry also supports tuple-like struct types and enum items:

```
pub struct MyStringWrapper(pub String);
```

You can access their inner values using pattern matching:

```
let MyStringWrapper(str) = wrapper;
println(str);
```

# Active patterns

You can define active patterns like in F#:

```
pattern (Even | Odd) (n: uint32) {
  if n % 2 == 0 {
    Even
  } else {
    Odd
  }
}
```
