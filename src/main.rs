//! The `rstory` binary. The real CLI arrives with spec 006; `daemon` is wired here for spec 004.

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => println!("rstory {}", env!("CARGO_PKG_VERSION")),
        Some("daemon") => {
            let rt = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(rt) => rt,
                Err(e) => {
                    eprintln!("rstory daemon: could not start the runtime: {e}");
                    return std::process::ExitCode::from(1);
                }
            };
            return std::process::ExitCode::from(
                rt.block_on(rstory::daemon::start::run_main(&args[1..])),
            );
        }
        _ => println!("usage: rstory [--version] | daemon [--backend x11|wayland]"),
    }
    std::process::ExitCode::SUCCESS
}
