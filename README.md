# cel-fmt

A code formatter for [Common Expression Language (CEL)](https://github.com/google/cel-spec) written in Rust.

[![CI](https://github.com/sevki/cel-fmt/actions/workflows/ci.yml/badge.svg)](https://github.com/sevki/cel-fmt/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

## Overview

`cel-fmt` is a fast, reliable formatter for CEL expressions. It parses CEL code using the [cel-rust](https://github.com/cel-rust/cel-rust) parser and reformats it with consistent style, similar to how `rustfmt` formats Rust code or `gofmt` formats Go code.

CEL is used extensively in Kubernetes for validation rules, admission policies, and custom resource definitions (CRDs). Having a standardized formatter helps maintain consistency across your CEL expressions.

## Features

- **Fast**: Written in Rust for maximum performance
- **Reliable**: Built on top of the battle-tested cel-rust parser
- **Configurable**: Customize indentation, line width, and other formatting options
- **CLI Tool**: Easy to integrate into your workflow
- **Format on stdin**: Pipe expressions directly to cel-fmt
- **File formatting**: Format CEL files in place or check formatting

## Installation

### From Source

```bash
git clone --recurse-submodules https://github.com/sevki/cel-fmt
cd cel-fmt
cargo install --path .
```

### From Releases

Download pre-built binaries from the [releases page](https://github.com/sevki/cel-fmt/releases).

## Usage

### Format from stdin

```bash
echo 'x>5&&y<10' | cel-fmt
# Output: x > 5 && y < 10
```

### Format a file

```bash
# Format and write back to file
cel-fmt myfile.cel

# Print formatted output without modifying file
cel-fmt --print myfile.cel

# Check if file is formatted (exit code 1 if not)
cel-fmt --check myfile.cel
```

### Command-line Options

```
Usage: cel-fmt [OPTIONS] [FILE]...

Arguments:
  [FILE]...  Input file(s) to format. If not specified, reads from stdin

Options:
  -c, --check                Check if files are formatted (don't modify)
  -w, --max-width <WIDTH>    Maximum line width [default: 80]
  -i, --indent <WIDTH>       Number of spaces per indentation level [default: 2]
      --use-tabs             Use tabs instead of spaces for indentation
      --no-trailing-comma    Don't add trailing commas
  -p, --print                Print the formatted output (don't modify files)
  -h, --help                 Print help
  -V, --version              Print version
```

## Examples

### Basic Expressions

**Input:**
```cel
1+2*3
```

**Output:**
```cel
1 + 2 * 3
```

### Lists and Maps

**Input:**
```cel
[1,2,3]
{"a":1,"b":2}
```

**Output:**
```cel
[1, 2, 3]
{"a": 1, "b": 2}
```

### Complex Kubernetes Validation

**Input:**
```cel
object.name.startsWith("kube-")&&object.namespace=="kube-system"
```

**Output:**
```cel
object.name.startsWith("kube-") && object.namespace == "kube-system"
```

### Multi-line Formatting

Long expressions are automatically wrapped for readability:

**Input:**
```cel
(has(metadata.labels) && "app" in metadata.labels) && (has(spec.template.spec.containers) && size(spec.template.spec.containers) > 0)
```

**Output (with `--max-width 60`):**
```cel
(has(metadata.labels) && "app" in metadata.labels) &&
  (has(spec.template.spec.containers) &&
    size(spec.template.spec.containers) > 0)
```

## Use Cases

### Kubernetes CRD Validation

Format CEL expressions in Custom Resource Definitions:

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
spec:
  validation:
    openAPIV3Schema:
      properties:
        spec:
          x-kubernetes-validations:
            - rule: "self.replicas >= 1 && self.replicas <= 100"
              message: "replicas must be between 1 and 100"
```

### ValidatingAdmissionPolicy

Format CEL expressions in admission policies:

```yaml
apiVersion: admissionregistration.k8s.io/v1
kind: ValidatingAdmissionPolicy
spec:
  validations:
    - expression: "object.spec.replicas >= 1"
      message: "replicas must be positive"
```

## Library Usage

You can also use cel-fmt as a library in your Rust projects:

```rust
use cel_fmt::{format_cel, FormatOptions};

fn main() {
    let source = "x>5&&y<10";
    let options = FormatOptions::default()
        .with_max_width(80)
        .with_indent_width(2);

    match format_cel(source, &options) {
        Ok(formatted) => println!("{}", formatted),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Configuration

Create a `.cel-fmt.toml` in your project root (coming soon):

```toml
max_width = 100
indent_width = 4
use_tabs = false
trailing_comma = true
```

## Development

### Building

```bash
git clone --recurse-submodules https://github.com/sevki/cel-fmt
cd cel-fmt
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Known Limitations

- **Comprehension Formatting**: Macros like `.map()`, `.filter()`, `.all()`, and `.exists()` are expanded by the parser into comprehension expressions, which are currently formatted as `<comprehension>`. Future versions will detect and format these back to their original macro form.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- Built on top of [cel-rust](https://github.com/cel-rust/cel-rust)
- CEL specification from [Google CEL](https://github.com/google/cel-spec)
- Inspired by rustfmt, gofmt, and prettier

## Related Projects

- [cel-rust](https://github.com/cel-rust/cel-rust) - CEL parser and interpreter for Rust
- [cel-go](https://github.com/google/cel-go) - Go implementation of CEL
- [cel-spec](https://github.com/google/cel-spec) - Common Expression Language specification
