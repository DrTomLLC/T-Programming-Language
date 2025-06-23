# T‑Lang CLI & UX Specification

This document defines the command‑line interface (CLI) and user experience (UX) guidelines for the T‑Lang compiler (`tlang`). It covers subcommands, flags, interactive behavior, output formatting, and plugin extension points.

---

## 1. Overview

The `tlang` binary is the primary entrypoint for all T‑Lang tooling. It must:

* Expose a consistent and discoverable set of subcommands.
* Provide sensible defaults and clear help text.
* Fail fast with actionable diagnostics on invalid input.
* Emit machine‑readable output when requested for IDE or automation integration.

---

## 2. Subcommands

| Command   | Description                                                        |
| --------- | ------------------------------------------------------------------ |
| `build`   | Compile one or more T‑Lang modules to object files or executables. |
| `run`     | Build then execute a T‑Lang program in a single step.              |
| `test`    | Discover and run unit & integration tests in T‑Lang projects.      |
| `fmt`     | Format T‑Lang source files according to style rules.               |
| `doc`     | Generate API documentation from T‑Lang source.                     |
| `init`    | Create a new T‑Lang project skeleton.                              |
| `plugin`  | Manage compiler plugins (list, install, remove, inspect).          |
| `backend` | List or configure codegen backends for native targets.             |
| `version` | Show compiler version and build metadata.                          |
| `help`    | Show usage information for `tlang` or a specific subcommand.       |

### 2.1. Example Usage

```bash
# Compile in debug mode (default)
tlang build

# Run with verbose logging:
tlang run --verbose

# Format all sources:
tlang fmt src/**/*.tlang
```

---

## 3. Global Flags

These flags apply to all subcommands (positioned before the subcommand name):

| Flag                | Shorthand | Description                                                                        |
| ------------------- | --------- | ---------------------------------------------------------------------------------- |
| `--verbose`         | `-v`      | Enable detailed logging (multiple levels supported).                               |
| `--quiet`           | `-q`      | Suppress non‑error output.                                                         |
| `--color <when>`    | —         | Control colored output: `auto` (default), `always`, `never`.                       |
| `--profile <name>`  | —         | Use a custom build profile (`debug`, `release`, etc.).                             |
| `--target <triple>` | —         | Cross‑compile for a specific target triple (e.g. `armv7-unknown-linux-gnueabihf`). |
| `--feature <name>`  | —         | Enable a named feature; can be repeated.                                           |
| `--json`            | —         | Output in JSON for editor or CI integration.                                       |

---

## 4. UX Principles

1. **Clarity**: Error messages and help text must be concise and instructive.
2. **Consistency**: Commands, flags, and output formats follow unified conventions.
3. **Progress Feedback**: Long‑running operations display progress bars or spinners.
4. **Non‑destructive Defaults**: By default, avoid overwriting files without confirmation.
5. **Accessibility**: Respect `NO_COLOR` and other environment‑driven preferences.

---

## 5. Error Reporting

* **Syntax & Type Errors**: Show file, line, column, and a one‑sentence summary.
* **Internal Errors**: Suggest filing a bug; include a unique error code.
* **Plugin Errors**: Report plugin name, version, and failure context.

### 5.1. Example

```text
error[E1001]: Unexpected token `}`
  --> src/main.tlang:42:5
   |
42 |     }
   |     ^ unexpected closing brace

help: To fix this, remove the extra `}` or add a matching `{` above.
```

---

## 6. Plugin Integration Hooks

T‑Lang supports plugins that interpose on compilation phases:

| Hook           | Phase           | Description                           |
| -------------- | --------------- | ------------------------------------- |
| `pre_parse`    | Before parsing  | Inspect or transform raw source text. |
| `post_parse`   | After parsing   | Validate AST, inject macros.          |
| `pre_lower`    | Before lowering | Hook into TIR lowering.               |
| `post_lower`   | After lowering  | Inject optimizations or analyses.     |
| `pre_codegen`  | Before codegen  | Adjust IR or select backends.         |
| `post_codegen` | After codegen   | Post‑process emitted artifacts.       |

Plugins register by returning a `#[no_mangle] pub extern "C" fn register(reg: &mut PluginRegistry)`.

---

<small>Document last updated: `{{date}}`</small>
