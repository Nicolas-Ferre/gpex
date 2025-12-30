# GPEx

[![MIT/Apache license](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/Nicolas-Ferre/gpex?tab=Apache-2.0-1-ov-file)
[![CI](https://github.com/Nicolas-Ferre/gpex/actions/workflows/ci.yml/badge.svg)](https://github.com/Nicolas-Ferre/gpex/actions/workflows/ci.yml)

GPEx is a programming language to run entire applications on the GPU.

It is particularly well suited for graphics applications like games.

## Experimental status

Before you consider using this language, please keep in mind that:

- It is developed by a single person in their spare time.
- The language is highly experimental: many important features have not yet been implemented, and
  frequent breaking changes are to be expected.

## Key features

- **GPU-native:** entire application logic runs on the GPU.
- **Data-race-safe:** potential data races are detected at compile time, preventing undefined
  behaviors.
- **Hot-reloadable:** code changes can be visualized without restarting the application.
- **Compiled as shaders:** `GPEx` is compiled ahead of time to WGSL, which is then
  compiled at application startup into GPU machine code.

## Supported platforms

- ![Windows supported](https://img.shields.io/badge/Windows-Supported-blue?logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyMSAyMSI+PHBhdGggZmlsbD0iI2YzNTMyNSIgZD0iTTAgMGgxMHYxMEgweiIvPjxwYXRoIGZpbGw9IiM4MWJjMDYiIGQ9Ik0xMSAwaDEwdjEwSDExeiIvPjxwYXRoIGZpbGw9IiMwNWE2ZjAiIGQ9Ik0wIDExaDEwdjEwSDB6Ii8+PHBhdGggZmlsbD0iI2ZmYmEwOCIgZD0iTTExIDExaDEwdjEwSDExeiIvPjwvc3ZnPg==)
- ![Linux supported](https://img.shields.io/badge/Linux-Supported-F4BC00?logo=linux)
- ![macOS supported](https://img.shields.io/badge/macOS-Supported-808080?logo=apple)
- ![Web planned](https://img.shields.io/badge/Web-Planned-d65600?logo=html5)
- ![Android planned](https://img.shields.io/badge/Android-Planned-008f11?logo=android)
