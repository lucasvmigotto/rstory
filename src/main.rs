//! The `rstory` binary. The real CLI arrives with spec 006.

fn main() {
    let arg = std::env::args().nth(1);
    match arg.as_deref() {
        Some("--version" | "-V") => println!("rstory {}", env!("CARGO_PKG_VERSION")),
        _ => println!("usage: rstory [--version]"),
    }
}
