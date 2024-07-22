#[allow(dead_code)]
fn original_main() {
    use clap::Parser;
    let args = upm::host::UpmArgs::parse();

    env_logger::init();

    let ret = if args.worker != 0 {
        upm::worker::run_as_worker(args.worker)
    } else {
        upm::host::run_as_controller(&args)
    };

    if let Err(e) = ret {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn main() -> anyhow::Result<()> {
    upm::tui::app()?;
    Ok(())
}
