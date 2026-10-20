# SysCore

> A highly performant backend solution for RAN Online web powered by Rust.

## Getting Started

First, run the development server:

```bash
cargo run
```

## Learn More

To learn more about SysCore, take a look at the following resources : WALA

## Coding Standards

### Readability

A primary characteristic for code is readability. The intent and meaning of the code should be visible to the reader. Being clear and unambiguous beats being clever and saving two lines of code. Write simple code. You and others who come back to this code over the coming decades want to be able to quickly understand it when debugging

### Naming

Try using a non-confusing naming scheme for your new functions and variable names. It does not necessarily have to mean that you should use the same as in other places of the code, just that the names should be logical, understandable and be named according to what they are used for. File-local functions should be made static. We like lower case names.

See the INTERNALS document on how we name non-exported library-global symbols

### Comments

Since we write Rust code, The following comments are for certain things

```rs

// This is for normal comment like questions and so on.

// Function Comments
/**
 *  Function Name
 *  What does it do?
 * 
 *  Parameters
 *  ParamType1 
 *  ParamType2
 */
```

### Long lines

Functions may never be wider than 79 columns and there are two reasons for maintaining this even in the modern era of large and high resolution screens

Narrower columns are easier to read than wide ones. There is a reason newspapers have used columns for decades or centuries.

Narrower columns allow developers to easier show multiple pieces of code next to each other in different windows. It allows two or three source code windows next to each other on the same screen - as well as multiple terminal and debugging windows

### Braces

In if/match/loop/for expressions, we write the open brace on the same line as the keyword and we then set the closing brace on the same indentation level as the initial keyword. Like this

```rs
if(age < 40) {
  /* clearly a youngster */
}
```

You may omit the braces if they would contain only a one-line statement

```rs
if(!x)
  continue;
```

For functions the opening brace should be on the function line

```rs
fn main(argc: i32) {
  return 1;
}
```
