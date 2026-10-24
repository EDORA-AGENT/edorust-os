#![no_std]
#![no_main]

use core::arch::asm;
use core::ptr::{addr_of, addr_of_mut, null_mut, read_volatile, write_volatile};

mod panic;

const LIMINE_COMMON_MAGIC_0: u64 = 0xc7b1dd30df4c8b88;
const LIMINE_COMMON_MAGIC_1: u64 = 0x0a82e883a194f07b;

// Limine base revision request.
#[used]
#[unsafe(link_section = ".limine_requests")]
static BASE_REVISION: [u64; 3] = [
    0xf9562b2d5c95a6c8,
    0x6a7b384944536bdc,
    3,
];

#[repr(C)]
struct FramebufferRequest {
    id: [u64; 4],
    revision: u64,
    response: *mut FramebufferResponse,
}

unsafe impl Sync for FramebufferRequest {}

#[repr(C)]
struct FramebufferResponse {
    revision: u64,
    framebuffer_count: u64,
    framebuffers: *mut *mut Framebuffer,
}

#[repr(C)]
struct Framebuffer {
    address: *mut u8,
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
    memory_model: u8,
    red_mask_size: u8,
    red_mask_shift: u8,
    green_mask_size: u8,
    green_mask_shift: u8,
    blue_mask_size: u8,
    blue_mask_shift: u8,
    _unused: [u8; 7],
    edid_size: u64,
    edid: *mut u8,
    mode_count: u64,
    modes: *mut *mut u8,
}

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest {
    id: [
        LIMINE_COMMON_MAGIC_0,
        LIMINE_COMMON_MAGIC_1,
        0x9d5827dcd881dd75,
        0xa3148604f6fab11b,
    ],
    revision: 0,
    response: null_mut(),
};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        let response_ptr =
            read_volatile(addr_of!(FRAMEBUFFER_REQUEST.response));

        if !response_ptr.is_null() {
            let response = &*response_ptr;

            if response.framebuffer_count > 0
                && !response.framebuffers.is_null()
            {
                let fb_ptr = read_volatile(response.framebuffers);

                if !fb_ptr.is_null() {
                    let fb = &*fb_ptr;

                    if fb.bpp == 32
                        && fb.memory_model == 1
                        && !fb.address.is_null()
                    {
                        clear_screen(fb, rgb(fb, 12, 18, 38));

                        let title = b"EDORA NOVA BOOTED";
                        let scale = 4usize;
                        let text_width = title.len() * 6 * scale;
                        let x = (fb.width as usize).saturating_sub(text_width) / 2;
                        let y = (fb.height as usize).saturating_sub(7 * scale) / 2;

                        draw_text(
                            fb,
                            x,
                            y,
                            title,
                            scale,
                            rgb(fb, 80, 220, 255),
                        );

                        draw_text(
                            fb,
                            x,
                            y + 40,
                            b"RUST KERNEL",
                            2,
                            rgb(fb, 220, 230, 245),
                        );
                    }
                }
            }
        }
    }

    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)); }
    }
}

unsafe fn clear_screen(fb: &Framebuffer, color: u32) {
    let width = fb.width as usize;
    let height = fb.height as usize;
    let pitch = fb.pitch as usize;

    for y in 0..height {
        for x in 0..width {
            let offset = y * pitch + x * 4;
            unsafe {
                write_volatile(
                    fb.address.add(offset).cast::<u32>(),
                    color,
                );
            }
        }
    }
}

fn channel(value: u8, bits: u8, shift: u8) -> u32 {
    if bits == 0 {
        return 0;
    }

    let max = (1u32 << bits) - 1;
    (((value as u32 * max) / 255) << shift)
}

fn rgb(fb: &Framebuffer, r: u8, g: u8, b: u8) -> u32 {
    channel(r, fb.red_mask_size, fb.red_mask_shift)
        | channel(g, fb.green_mask_size, fb.green_mask_shift)
        | channel(b, fb.blue_mask_size, fb.blue_mask_shift)
}

fn glyph(c: u8) -> [u8; 7] {
    match c {
        b'A' => [14, 17, 17, 31, 17, 17, 17],
        b'B' => [30, 17, 17, 30, 17, 17, 30],
        b'D' => [30, 17, 17, 17, 17, 17, 30],
        b'E' => [31, 16, 16, 30, 16, 16, 31],
        b'N' => [17, 25, 21, 19, 17, 17, 17],
        b'O' => [14, 17, 17, 17, 17, 17, 14],
        b'R' => [30, 17, 17, 30, 20, 18, 17],
        b'T' => [31, 4, 4, 4, 4, 4, 4],
        b'U' => [17, 17, 17, 17, 17, 17, 14],
        b'V' => [17, 17, 17, 17, 17, 10, 4],
        b' ' => [0; 7],
        _ => [31, 17, 5, 4, 0, 4, 0],
    }
}

unsafe fn draw_text(
    fb: &Framebuffer,
    x: usize,
    y: usize,
    text: &[u8],
    scale: usize,
    color: u32,
) {
    for (i, &ch) in text.iter().enumerate() {
        let bitmap = glyph(ch);
        let char_x = x + i * 6 * scale;

        for (row, bits) in bitmap.iter().enumerate() {
            for col in 0..5usize {
                if bits & (1 << (4 - col)) == 0 {
                    continue;
                }

                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = char_x + col * scale + sx;
                        let py = y + row * scale + sy;

                        if px < fb.width as usize
                            && py < fb.height as usize
                        {
                            let offset =
                                py * fb.pitch as usize + px * 4;

                            unsafe {
                                write_volatile(
                                    fb.address.add(offset).cast::<u32>(),
                                    color,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
