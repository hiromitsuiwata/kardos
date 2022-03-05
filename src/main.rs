#![no_main]
#![no_std]
#![feature(abi_efiapi)]

use core::fmt::Write;
use log::info;
use uefi::prelude::*;
use uefi::table::runtime::ResetType;
use uefi::ResultExt;

#[entry]
fn efi_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap_success();

    system_table.stdout().reset(false).unwrap_success();

    writeln!(system_table.stdout(), "[writeln] Hello, world from writeln").unwrap();
    info!("[info] Hello, world from info");

    // Status::SUCCESS
    // Stalls the processor for an amount of time. 300 seconds.
    system_table.boot_services().stall(300_000_000);
    system_table.stdout().reset(false).unwrap_success();
    system_table
        .runtime_services()
        .reset(ResetType::Shutdown, Status::SUCCESS, None);
}
