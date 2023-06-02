# Madara: Coding Principles and Best Practices Guide

## Table of Contents

- [Madara: Coding Principles and Best Practices Guide](#madara-coding-principles-and-best-practices-guide)
  - [Table of Contents](#table-of-contents)
  - [1. Introduction](#1-introduction)
  - [2. Clean Code](#2-clean-code)
  - [3. Principles](#3-principles)
    - [3.1 KISS: Keep It Simple, Stupid](#31-kiss-keep-it-simple-stupid)
    - [3.2 YAGNI: You Aren't Gonna Need It](#32-yagni-you-arent-gonna-need-it)
    - [3.3 DRY: Don't Repeat Yourself](#33-dry-dont-repeat-yourself)
    - [3.4 SRP: Single Responsibility Principle](#34-srp-single-responsibility-principle)
  - [4. Conclusion](#4-conclusion)

## 1. Introduction

This document outlines the principles and practices we strive to uphold in the
development of the Madara project. We aim to create a codebase that is clean,
maintainable, and efficient. To achieve this, we draw inspiration from
principles laid out in various reference books, most notably Clean Code: A
Handbook of Agile Software Craftsmanship.

## 2. Clean Code

A core philosophy of our project is to create clean, readable, and maintainable
code. Clean code allows developers to understand the code better, reduce bugs,
and ease the maintenance process.

In the spirit of Clean Code, we strive to:

- Write meaningful names
- Write small functions doing one thing
- Minimize the number of function arguments
- Avoid side effects
- Write DRY code
- Write unit tests
- Write meaningful comments

An example of clean code in Rust could be:

```rust
pub struct Circle {
    radius: f64,
}

impl Circle {
    pub fn new(radius: f64) -> Circle {
        Circle { radius }
    }

    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}
```

This simple `Circle` struct and its implementation is clean: it's
straightforward, self-describing, and only has the necessary functionality.

## 3. Principles

### 3.1 KISS: Keep It Simple, Stupid

The KISS principle states that simplicity should be a key goal in design and
unnecessary complexity should be avoided. This will make your code more readable
and maintainable.

Here's an example of adhering to the KISS principle:

```rust
// Complex way
fn calculate_sum(numbers: &[i32]) -> i32 {
    let mut sum = 0;
    for i in 0..numbers.len() {
        sum += numbers[i];
    }
    sum
}

// KISS way
fn calculate_sum(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}
```

### 3.2 YAGNI: You Aren't Gonna Need It

The YAGNI principle emphasizes not adding functionality until it is necessary.
This reduces complexity and increases code maintainability.

```rust
struct User {
    id: i32,
    name: String,
    // age: i32, // YAGNI, don't add it until it's necessary
}

impl User {
    fn new(id: i32, name: String) -> User {
        User { id, name }
    }
}
```

### 3.3 DRY: Don't Repeat Yourself

The DRY principle is aimed at reducing repetition. It helps to lower the chance
of bugs and makes the code more maintainable.

```rust
// Violates DRY
fn add_ten(num: i32) -> i32 {
    num + 10
}

fn add_twenty(num: i32) -> i32 {
    num + 20
}

// Follows DRY
fn add_n(num: i32, n: i32) -> i32 {
    num + n
}
```

In this case, instead of having separate functions to add ten or twenty to a
number, we can have a general function to add any integer to another.

### 3.4 SRP: Single Responsibility Principle

SRP suggests a component of software (a module, a class, or a function) should
have one, and only one, reason to change.

Let's look at a struct that violates the SRP:

```rust
pub struct Report {
    title: String,
    data: Vec<String>,
}

impl Report {
    pub fn new(title: String, data: Vec<String>) -> Report {
        Report { title, data }
    }

    pub fn print(&self) {
        println!("Title: {}", self.title);
        for line in &self.data {
            println!("{}", line);
        }
    }

    pub fn format(&mut self) {
        self.data = self.data.iter().map(|line| format!("{}\n", line)).collect();
    }
}
```

Here, the `Report` struct is responsible for data handling (`new`, `format`) and
for output (`print`). This violates the Single Responsibility Principle.

A better approach is to separate these responsibilities into different structs:

```rust
pub struct Report {
    title: String,
    data: Vec<String>,
}

impl Report {
    pub fn new(title: String, data: Vec<String>) -> Report {
        Report { title, data }
    }

    pub fn format(&mut self) {
        self.data = self.data.iter().map(|line| format!("{}\n", line)).collect();
    }
}

pub struct ReportPrinter {
    report: Report,
}

impl ReportPrinter {
    pub fn new(report: Report) -> ReportPrinter {
        ReportPrinter { report }
    }

    pub fn print(&self) {
        println!("Title: {}", self.report.title);
        for line in &self.report.data {
            println!("{}", line);
        }
    }
}
```

Now, `Report` is responsible for data handling, and `ReportPrinter` is
responsible for output. Each struct now has a single responsibility.

## 4. Conclusion

This document laid out the core coding principles we strive to uphold in the
Madara project. Remember, these are principles, not strict rules. They are meant
to guide us towards producing a clean, maintainable, and efficient codebase. The
purpose is not to create perfect code, but to aim for better code every day.
Let's write some clean and beautiful Rust code together!
