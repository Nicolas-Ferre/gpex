## Description

Add `?` as a wildcard function parameter type.

The wildcard type is useless without the `typeof(expr)` operator, so this operator is also part of
the issue.

## Dependencies

- #170

## Details

### Wildcard type

- `?` wildcard is only accepted as a function parameter type.
- Multiple wildcard parameters infer their types independently.
- Function body validation is performed without considering actual calls of this function.
- When multiple function overloads match, keep the existing source and import search order.

```zig,gpex
fn identity(input: ?) -> typeof(input) {
    return input;
}

fn forward(value: ?) -> typeof(value) {
    return identity(value);
}

var _result1 = forward(123); // expected: 123
var _result2 = forward(1.2); // expected: 1.2
```

```zig,gpex
fn invalid_fn(param1: ?, param2: ?) -> typeof(param2) {
    return param1; // compilation error because `param1` type is different than `param2` type
}
```

### Constness

- Wildcard parameters can be qualified with `const`.

```zig,gpex
const fn identity(value: const ?) -> typeof(value) {
    return value;
}

const _RESULT1 = identity(123); // expected: 123
const _RESULT2 = identity(1.2); // expected: 1.2

var variable = 0;
var _error = identity(variable); // compilation error because runtime value is not allowed
```

### `typeof` function

- `typeof(_)` accept any valid expression, including runtime values.
- `typeof(_)` returns the expression type as a constant `typeref` value.

## Steps

- Add `?` wildcard function parameter type.
- Add `typeof` operator.
- Improve `?` wildcard tests.
