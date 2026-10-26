# TeeChart for Rust

Charts from Rust using the TeeChart native library.

```toml
[dependencies]
teechart = "0.9"
```

```rust
use teechart::Chart;

fn main() -> Result<(), teechart::Error> {
    Chart::new()
        .y(&[2.5, 3.2, 2.8, 4.1])
        .texts(&["One", "Two", "Three", "Four"])
        .style("pie")
        .title("TeeChart for Rust")
        .render()
}
```

Run the example: `cargo run --example simple`

## Output: 

<img width="998" height="467" alt="image" src="https://github.com/user-attachments/assets/7b03705c-9e6a-48b4-a737-cc2d601faf77" />

## Native library

Bundled in `libs/` (`win64/teechart.dll`, `linux64/libteechart.so`, `macos/libteechart.dylib`).
Search order: `TEECHART_LIB` environment variable, next to the executable, `libs/<platform>/`.

See also: [TeeChart for R](https://github.com/Steema/TeeChart-for-R) · [TeeChart for Python](https://github.com/Steema/TeeChart-for-Python)
