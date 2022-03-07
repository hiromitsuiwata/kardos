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
use uefi::proto::media::file::{Directory, File, FileAttribute, FileHandle, FileMode, RegularFile};
use uefi::table::boot::MemoryDescriptor;
use uefi::table::runtime::ResetType;
use uefi::ResultExt;

#[entry]
fn efi_main(image_handle: Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).unwrap_success();
    system_table.stdout().reset(false).unwrap_success();
    writeln!(system_table.stdout(), "start efi_main...").unwrap();

    // メモリマップ書き込み先のファイルを作成
    info!("create a file");
    let file = create_file(image_handle, &system_table);

    // メモリマップ書き込み
    info!("write memory map");
    write_memory_descriptor(&system_table, file);

    // シャットダウンまで10秒待機
    info!("complete");
    writeln!(system_table.stdout(), "waiting until shutdown...").unwrap();
    system_table.boot_services().stall(10_000_000);
    system_table.stdout().reset(false).unwrap_success();
    // シャットダウン
    system_table
        .runtime_services()
        .reset(ResetType::Shutdown, Status::SUCCESS, None);
}

fn create_file(image_handle: Handle, system_table: &SystemTable<Boot>) -> RegularFile {
    let file_system = unsafe {
        &mut *system_table
            .boot_services()
            .get_image_file_system(image_handle)
            .unwrap_success()
            .interface
            .get()
    };

    let mut directory: Directory = file_system.open_volume().unwrap_success();

    let file_handle: FileHandle = directory
        .open(
            "memory_map.txt",
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .unwrap_success();

    //RegularFile
    let file: RegularFile;
    unsafe {
        file = RegularFile::new(file_handle);
    }
    return file;
}

fn write_memory_descriptor(system_table: &SystemTable<Boot>, mut file: RegularFile) {
    let size = system_table.boot_services().memory_map_size().map_size
        + 8 * mem::size_of::<MemoryDescriptor>();
    let mut buffer = vec![0; size];
    let (_, descriptors) = system_table
        .boot_services()
        .memory_map(&mut buffer)
        .unwrap_success();

    file.write("Index, Type, Type(name), PhysicalStart, NumberOfPages, Attribute\n".as_bytes())
        .unwrap_success();
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
    }
    file.flush().unwrap_success();
}
