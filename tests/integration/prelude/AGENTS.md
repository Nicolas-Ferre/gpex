# Prelude test guidelines

- Equality and comparison tests should cover both true and false results.
- Avoid repeating the tested type in test variable and constant names when the surrounding file
  already identifies it.
- Keep variants of the same operation adjacent so their behavior and edge cases are easy to
  compare.
- Prelude operations should have matching runtime and compile-time test coverage, including edge
  cases.
