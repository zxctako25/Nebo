#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub fn run_if_monitor() -> anyhow::Result<bool> {
    let mut args = std::env::args().skip(1);
    if args.next().as_deref() != Some("--resource-monitor") {
        return Ok(false);
    }
    let pid = args
        .next()
        .ok_or_else(|| anyhow::anyhow!("resource monitor PID is missing"))?
        .parse::<u32>()?;
    windows::run(pid);
    Ok(true)
}

#[cfg(not(windows))]
pub fn run_if_monitor() -> anyhow::Result<bool> {
    Ok(false)
}

#[cfg(windows)]
pub fn spawn(pid: u32) -> anyhow::Result<()> {
    windows::spawn(pid)
}

#[cfg(not(windows))]
pub fn spawn(_pid: u32) -> anyhow::Result<()> {
    Ok(())
}
