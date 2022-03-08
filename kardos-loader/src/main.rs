#![no_main]
#![no_std]
#![feature(abi_efiapi)]
#![feature(vec_into_raw_parts)]

#[macro_use]
extern crate alloc;

use core::fmt::Write;
use core::{mem, slice};
use goblin::elf;
use log::info;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, PixelFormat};
use uefi::proto::media::file::{
    Directory, File, FileAttribute, FileHandle, FileInfo, FileMode, RegularFile,
};
use uefi::table::boot::{AllocateType, MemoryDescriptor, MemoryType};
use uefi::table::runtime::ResetType;
use uefi::ResultExt;

const EFI_PAGE_SIZE: usize = 0x1000;

#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum MyPixelFormat {
    Rgb,
    Bgr,
}

#[repr(C)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct FrameBuffer {
    pub frame_buffer: *mut u8,
    pub stride: u32,
    pub resolution: (u32, u32), // (horizontal, vertical)
    pub format: MyPixelFormat,
}

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

    // カーネル読み込み
    info!("load kernel");
    load_kernel(image_handle, &system_table);

    // シャットダウンまで120秒待機
    info!("complete");
    writeln!(system_table.stdout(), "waiting until shutdown...").unwrap();
    system_table.boot_services().stall(120_000_000);
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

    let mut root_dir: Directory = file_system.open_volume().unwrap_success();

    let file_handle: FileHandle = root_dir
        .open(
            "memory_map.csv",
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

fn load_kernel(image_handle: Handle, system_table: &SystemTable<Boot>) {
    let file_system = unsafe {
        &mut *system_table
            .boot_services()
            .get_image_file_system(image_handle)
            .unwrap_success()
            .interface
            .get()
    };

    let mut root_dir: Directory = file_system.open_volume().unwrap_success();

    let file_handle: FileHandle = root_dir
        .open(
            "kernel.elf",
            FileMode::CreateReadWrite,
            FileAttribute::empty(),
        )
        .unwrap_success();

    //RegularFile
    let mut file: RegularFile;
    unsafe {
        file = RegularFile::new(file_handle);
    }

    let size = file
        .get_boxed_info::<FileInfo>()
        .unwrap_success()
        .file_size() as usize;

    info!("elf size: {}", size);
    let mut buf = vec![0; size];
    file.read(&mut buf).unwrap_success();

    info!("elf header: {:?}", &buf[0..20]);
    let elf = elf::Elf::parse(&buf).expect("Failed to parse ELF");

    info!("load elf");

    let mut dest_start = usize::MAX;
    let mut dest_end = 0;
    for ph in elf.program_headers.iter() {
        if ph.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        dest_start = dest_start.min(ph.p_vaddr as usize);
        dest_end = dest_end.max((ph.p_vaddr + ph.p_memsz) as usize);
    }

    system_table
        .boot_services()
        .allocate_pages(
            AllocateType::Address(dest_start),
            MemoryType::LOADER_DATA,
            (dest_end - dest_start + EFI_PAGE_SIZE - 1) / EFI_PAGE_SIZE,
        )
        .expect_success("Failed to allocate pages for kernel");

    for ph in elf.program_headers.iter() {
        if ph.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        let ofs = ph.p_offset as usize;
        let fsize = ph.p_filesz as usize;
        let msize = ph.p_memsz as usize;
        let dest = unsafe { slice::from_raw_parts_mut(ph.p_vaddr as *mut u8, msize) };
        dest[..fsize].copy_from_slice(&buf[ofs..ofs + fsize]);
        dest[fsize..].fill(0);
    }

    info!("load entry_point");
    let entry_point: extern "sysv64" fn(&FrameBuffer) = unsafe { mem::transmute(elf.entry) };

    info!("frame_buffer");
    let gop = system_table
        .boot_services()
        .locate_protocol::<GraphicsOutput>()
        .unwrap_success();
    let gop = unsafe { &mut *gop.get() };
    let frame_buffer = FrameBuffer {
        frame_buffer: gop.frame_buffer().as_mut_ptr(),
        stride: gop.current_mode_info().stride() as u32,
        resolution: (
            gop.current_mode_info().resolution().0 as u32,
            gop.current_mode_info().resolution().1 as u32,
        ),
        format: match gop.current_mode_info().pixel_format() {
            PixelFormat::Rgb => MyPixelFormat::Rgb,
            PixelFormat::Bgr => MyPixelFormat::Bgr,
            f => panic!("Unsupported pixel format: {:?}", f),
        },
    };

    info!("invoke entry_point");
    entry_point(&frame_buffer);
}
