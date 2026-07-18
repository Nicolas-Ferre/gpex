# Prelude guidelines

- Keep prelude function signatures on one line when they fit, including the return type and
  `= compilerimpl`.
- Every prelude docstring should include an "# Examples" section.
- Show each function example's expected result in an inline comment.
- Document an example for every supported type when a function is available only for a limited set
  of generic types, grouping related examples in one code block.
- Describe boolean-returning functions with "Returns whether ...".
- Keep docstrings complete but concise and refer explicitly to every parameter by name.
