#![cfg_attr(target_vendor = "uwp", windows_subsystem = "windows")]

#[cfg(target_vendor = "uwp")]
mod uwp;

#[cfg(target_vendor = "uwp")]
fn main() -> windows::core::Result<()> {
    uwp::run()
}

#[cfg(not(target_vendor = "uwp"))]
fn main() {
    eprintln!("The application requires a UWP host.");
}
