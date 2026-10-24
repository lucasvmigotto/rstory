//! The `rstory` binary. `daemon` is started here (the `cli` module may not depend on the
//! daemon); every other command is `rstory::cli::run`.

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("daemon") {
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
    std::process::ExitCode::from(rstory::cli::run(&args))
}
