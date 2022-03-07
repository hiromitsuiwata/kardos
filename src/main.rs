#![no_main]
#![no_std]
#![feature(abi_efiapi)]
#![feature(vec_into_raw_parts)]

#[macro_use]
extern crate alloc;

use core::fmt::Write;
use core::mem;
use log::info;
use uefi::prelude::*;
use uefi::proto::media::file::{
    Directory, File, FileAttribute, FileInfo, FileMode, FileType, RegularFile,
};
use uefi::table::boot::{MemoryDescriptor, ScopedProtocol};
use uefi::table::runtime::ResetType;
use uefi::ResultExt;

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap_success();

    system_table.stdout().reset(false).unwrap_success();

    writeln!(system_table.stdout(), "[writeln] Hello, world from writeln").unwrap();
    info!("[info] Hello, world from info");

    let size = system_table.boot_services().memory_map_size().map_size
        + 8 * mem::size_of::<MemoryDescriptor>();
    let mut buffer = vec![0; size];
    let (_, descriptors) = system_table
        .boot_services()
        .memory_map(&mut buffer)
        .unwrap_success();

    let file_handle = unsafe {
        &mut *system_table
            .boot_services()
            .get_image_file_system(image_handle)
            .unwrap_success()
            .interface
            .get()
    }
    .open_volume()
    .unwrap_success()
    .open(
        "memory_map.txt",
        FileMode::CreateReadWrite,
        FileAttribute::empty(),
    )
    .unwrap_success();

    //RegularFile
    let mut file: RegularFile;
    unsafe {
        file = RegularFile::new(file_handle);
    }

    for (i, d) in descriptors.enumerate() {
        let line = format!(
            "{}, {:x}, {:?}, {:08x}, {:x}, {:x}\n",
            i,
            d.ty.0,
            d.ty,
            d.phys_start,
            d.page_count,
            d.att.bits() & 0xfffff
        );
        file.write(line.as_bytes()).unwrap_success();
        write!(system_table.stdout(), r#"{}"#, line).unwrap();
    }
    file.flush().unwrap_success();

    // Status::SUCCESS
    // Stalls the processor for an amount of time. 300 seconds.
    system_table.boot_services().stall(300_000_000);
    system_table.stdout().reset(false).unwrap_success();
    system_table
        .runtime_services()
        .reset(ResetType::Shutdown, Status::SUCCESS, None);
}
