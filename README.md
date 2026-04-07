# MyLang Parser

Parser for the MyLang programming language written in Rust.

## Building

```bash
cargo build
```

## Running

```bash
cargo run -- <source_file> [-o|--output <output_file>]
```

### Arguments

- `<source_file>` - Path to MyLang source file
- `-o, --output <file>` - Output file for Mermaid diagram (default: stdout)

### Example: Hello World

Run:
```bash
cargo run -- examples/hello_world/input.mylang -o examples/hello_world/output.mmd
```

## Testing

```bash
cargo test
```
