#![no_main]
#![no_std]
#![feature(abi_efiapi)]
#![feature(vec_into_raw_parts)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use core::fmt::Write;
use core::{mem, slice};
use goblin::elf;
use log::info;
use uefi::prelude::*;
use uefi::proto::console::gop::{GraphicsOutput, Mode, ModeInfo, PixelFormat};
use uefi::proto::media::file::{
    Directory, File, FileAttribute, FileHandle, FileInfo, FileMode, RegularFile,
};
use uefi::table::boot::{AllocateType, MemoryDescriptor, MemoryType};
use uefi::table::runtime::ResetType;
use uefi::table::Runtime;
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
    let entry_point_addr = load_kernel(image_handle, &system_table);

    // ELFのヘッダーのエントリーポイントのアドレスを関数ポインタとして扱う
    // その関数はFrameBufferを引数とする
    let entry_point: extern "sysv64" fn(&FrameBuffer) = unsafe { mem::transmute(entry_point_addr) };

    // 画面出力のためのFrameBufferオブジェクトを作成する
    let graphics_output_unsafe = system_table
        .boot_services()
        .locate_protocol::<GraphicsOutput>()
        .unwrap_success();
    let graphics_output = unsafe { &mut *graphics_output_unsafe.get() };

    // 他のモードを取得する
    let modes = graphics_output.modes();
    for (i, mode) in modes.enumerate() {
        let m: Mode = mode.unwrap();
        let (horizontal, vertical) = m.info().resolution();
        info!("{}, horizontal: {}, vertical: {}", i, horizontal, vertical);
    }

    // ローダーの挙動を目で見るために3秒待機
    system_table.boot_services().stall(3_000_000);

    // 指定した解像度を持つモードへ変更
    let m = graphics_output.modes().nth(18).unwrap().unwrap();
    graphics_output.set_mode(&m).unwrap_success();

    // カーネルに渡すフレームバッファを作成
    let frame_buffer = FrameBuffer {
        frame_buffer: graphics_output.frame_buffer().as_mut_ptr(),
        stride: graphics_output.current_mode_info().stride() as u32,
        resolution: (
            graphics_output.current_mode_info().resolution().0 as u32,
            graphics_output.current_mode_info().resolution().1 as u32,
        ),
        format: match graphics_output.current_mode_info().pixel_format() {
            PixelFormat::Rgb => MyPixelFormat::Rgb,
            PixelFormat::Bgr => MyPixelFormat::Bgr,
            f => panic!("Unsupported pixel format: {:?}", f),
        },
    };

    // ブートサービスの停止
    let enough_mmap_size = system_table.boot_services().memory_map_size().map_size
        + 8 * mem::size_of::<MemoryDescriptor>();
    let mut mmap_buf = vec![0; enough_mmap_size];
    let (system_table, _) = system_table
        .exit_boot_services(image_handle, &mut mmap_buf[..])
        .expect_success("Failed to exit boot services");
    mem::forget(mmap_buf);

    // カーネル呼び出し
    entry_point(&frame_buffer);

    info!("complete");
    // シャットダウン
    unsafe {
        system_table
            .runtime_services()
            .reset(ResetType::Shutdown, Status::SUCCESS, None);
    }
}

fn change_resolution(graphics_output: GraphicsOutput, image_handle: Handle) {}

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

/// 指定したパスのファイルのRegularFileを取得する
fn get_file(image_handle: Handle, system_table: &SystemTable<Boot>, filepath: &str) -> RegularFile {
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
        .open(&filepath, FileMode::CreateReadWrite, FileAttribute::empty())
        .unwrap_success();

    //RegularFile
    let file: RegularFile;
    unsafe {
        file = RegularFile::new(file_handle);
    }
    file
}

fn size_of(file: &mut RegularFile) -> usize {
    let size = file
        .get_boxed_info::<FileInfo>()
        .unwrap_success()
        .file_size() as usize;
    size
}

fn read_file_to_vec(file: &mut RegularFile) -> Vec<u8> {
    let size = size_of(file);
    info!("elf size: {}", size);

    let mut buf = vec![0; size];
    file.read(&mut buf).unwrap_success();
    buf
}

fn load_kernel(image_handle: Handle, system_table: &SystemTable<Boot>) -> usize {
    let mut file = get_file(image_handle, system_table, "kernel.elf");

    let buf = read_file_to_vec(&mut file);
    info!("elf header: {:?}", &buf[0..16]);

    info!("load elf");
    let elf = elf::Elf::parse(&buf).expect("Failed to parse ELF");

    // ELFの複数のプログラムヘッダのなかのLOADセグメントの先頭と末尾のアドレスを探す
    let mut dest_start = usize::MAX;
    let mut dest_end = 0;
    for program_header in elf.program_headers.iter() {
        if program_header.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        dest_start = dest_start.min(program_header.p_vaddr as usize);
        dest_end = dest_end.max((program_header.p_vaddr + program_header.p_memsz) as usize);
    }

    // dest_startがlldの--image-baseオプションで指定したアドレス0x100000番地になっているはず
    info!("dest_start: {:x}", dest_start);
    info!("dest_end: {:x}", dest_end);
    info!(
        "page_size: {:x}",
        (dest_end - dest_start + EFI_PAGE_SIZE - 1) / EFI_PAGE_SIZE
    );

    // コピー先となるメモリ領域を確保する
    // lldの--image-baseオプションで指定したアドレス（メモリマップから十分な大きさを持つCONVENTIONALの領域）へアドレス決め打ちで配置したい
    system_table
        .boot_services()
        .allocate_pages(
            AllocateType::Address(dest_start),
            MemoryType::LOADER_DATA,
            (dest_end - dest_start + EFI_PAGE_SIZE - 1) / EFI_PAGE_SIZE,
        )
        .expect_success("Failed to allocate pages for kernel");

    // ELFの仮想アドレスへmemcopyでコピーする
    for program_header in elf.program_headers.iter() {
        if program_header.p_type != elf::program_header::PT_LOAD {
            continue;
        }
        let ofs = program_header.p_offset as usize;
        let fsize = program_header.p_filesz as usize;
        let msize = program_header.p_memsz as usize;
        // 指定したアドレスに配置されるスライスを作る
        let dest = unsafe { slice::from_raw_parts_mut(program_header.p_vaddr as *mut u8, msize) };
        // 作ったスライスにELFファイルを読み込んだメモリ領域のデータをコピーする
        dest[..fsize].copy_from_slice(&buf[ofs..ofs + fsize]);
        dest[fsize..].fill(0);
    }

    // ELFのヘッダーのエントリーポイントのアドレスを関数ポインタとして扱う
    // その関数はFrameBufferを引数とする
    //let entry_point: extern "sysv64" fn(&FrameBuffer) = unsafe { mem::transmute(elf.entry) };

    // info!("exit boot services");
    // let enough_mmap_size = system_table.boot_services().memory_map_size().map_size
    //     + 8 * mem::size_of::<MemoryDescriptor>();
    // let mut mmap_buf = vec![0; enough_mmap_size];
    // &system_table.exit_boot_services(image_handle, &mut mmap_buf[..]);
    // mem::forget(mmap_buf);

    //info!("invoke entry_point");
    //(entry_point, &frame_buffer);

    elf.entry as usize
}
