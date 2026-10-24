#![no_std]
#![no_main]

use core::arch::asm;
use core::ptr::{addr_of, null_mut, read_volatile, write_volatile};

mod panic;
mod memory;

const MAGIC0: u64 = 0xc7b1dd30df4c8b88;
const MAGIC1: u64 = 0x0a82e883a194f07b;

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
    id: [MAGIC0, MAGIC1, 0x9d5827dcd881dd75, 0xa3148604f6fab11b],
    revision: 0,
    response: null_mut(),
};

#[repr(C)]
struct MemoryMapRequest {
    id: [u64; 4],
    revision: u64,
    response: *mut memory::MemoryMapResponse,
}
unsafe impl Sync for MemoryMapRequest {}

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest {
    id: [MAGIC0, MAGIC1, 0x67cf3d9d378a806f, 0xe304acdfc50c3c62],
    revision: 0,
    response: null_mut(),
};

const BG: (u8, u8, u8) = (9, 14, 27);
const DARK: (u8, u8, u8) = (5, 8, 18);
const PANEL: (u8, u8, u8) = (17, 25, 43);
const PANEL2: (u8, u8, u8) = (22, 33, 53);
const CYAN: (u8, u8, u8) = (53, 220, 245);
const WHITE: (u8, u8, u8) = (230, 239, 250);
const MUTED: (u8, u8, u8) = (132, 153, 179);
const GREEN: (u8, u8, u8) = (74, 222, 128);
const PURPLE: (u8, u8, u8) = (167, 139, 250);
const RED: (u8, u8, u8) = (248, 113, 113);

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        let mm = read_volatile(addr_of!(MEMORY_MAP_REQUEST.response));
        if !mm.is_null() {
            let _ = memory::init(mm);
        }

        let response = read_volatile(addr_of!(FRAMEBUFFER_REQUEST.response));
        if !response.is_null() {
            let response = &*response;
            if response.framebuffer_count > 0 && !response.framebuffers.is_null() {
                let fb_ptr = read_volatile(response.framebuffers);
                if !fb_ptr.is_null() {
                    let fb = &*fb_ptr;
                    if fb.bpp == 32 && fb.memory_model == 1 && !fb.address.is_null() {
                        draw_splash(fb, b"INITIALIZING DISPLAY", 20);
                        draw_splash(fb, b"CHECKING MEMORY MAP", 45);
                        draw_splash(fb, b"STARTING PS2 KEYBOARD", 70);
                        draw_splash(fb, b"STARTING DESKTOP", 90);

                        let mut selected: usize = 0;
                        let mut page: Option<usize> = None;
                        render(fb, selected, page);

                        loop {
                            if let Some(key) = read_key() {
                                match key {
                                    0x48 | b'w' | b'W' => {
                                        if page.is_none() {
                                            selected = selected.saturating_sub(1);
                                            render(fb, selected, page);
                                        }
                                    }
                                    0x50 | b's' | b'S' => {
                                        if page.is_none() {
                                            selected = (selected + 1).min(3);
                                            render(fb, selected, page);
                                        }
                                    }
                                    0x1C => {
                                        page = Some(selected);
                                        render(fb, selected, page);
                                    }
                                    0x01 | b'h' | b'H' => {
                                        page = None;
                                        selected = 0;
                                        render(fb, selected, page);
                                    }
                                    b'r' | b'R' => render(fb, selected, page),
                                    _ => {}
                                }
                            }
                            // Polling remains functional even if IRQs are not enabled.
                            asm!("pause", options(nomem, nostack, preserves_flags));
                        }
                    }
                }
            }
        }
    }

    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)); }
    }
}

fn color(fb: &Framebuffer, rgb: (u8, u8, u8)) -> u32 {
    channel(rgb.0, fb.red_mask_size, fb.red_mask_shift)
        | channel(rgb.1, fb.green_mask_size, fb.green_mask_shift)
        | channel(rgb.2, fb.blue_mask_size, fb.blue_mask_shift)
}

fn channel(value: u8, bits: u8, shift: u8) -> u32 {
    if bits == 0 || bits > 8 { return 0; }
    let max = (1u32 << bits) - 1;
    (((value as u32 * max) / 255) << shift)
}

unsafe fn pixel(fb: &Framebuffer, x: usize, y: usize, c: u32) {
    if x >= fb.width as usize || y >= fb.height as usize { return; }
    let offset = y * fb.pitch as usize + x * 4;
    unsafe { write_volatile(fb.address.add(offset).cast::<u32>(), c); }
}

unsafe fn rect(fb: &Framebuffer, x: usize, y: usize, w: usize, h: usize, c: u32) {
    let max_x = x.saturating_add(w).min(fb.width as usize);
    let max_y = y.saturating_add(h).min(fb.height as usize);
    for py in y.min(max_y)..max_y {
        for px in x.min(max_x)..max_x {
            unsafe { pixel(fb, px, py, c); }
        }
    }
}

unsafe fn fill(fb: &Framebuffer, c: u32) {
    unsafe { rect(fb, 0, 0, fb.width as usize, fb.height as usize, c); }
}

fn glyph(c: u8) -> [u8; 7] {
    match c.to_ascii_uppercase() {
        b'A'=>[14,17,17,31,17,17,17], b'B'=>[30,17,17,30,17,17,30],
        b'C'=>[14,17,16,16,16,17,14], b'D'=>[30,17,17,17,17,17,30],
        b'E'=>[31,16,16,30,16,16,31], b'F'=>[31,16,16,30,16,16,16],
        b'G'=>[14,17,16,23,17,17,15], b'H'=>[17,17,17,31,17,17,17],
        b'I'=>[14,4,4,4,4,4,14], b'J'=>[7,2,2,2,18,18,12],
        b'K'=>[17,18,20,24,20,18,17], b'L'=>[16,16,16,16,16,16,31],
        b'M'=>[17,27,21,21,17,17,17], b'N'=>[17,25,21,19,17,17,17],
        b'O'=>[14,17,17,17,17,17,14], b'P'=>[30,17,17,30,16,16,16],
        b'Q'=>[14,17,17,17,21,18,13], b'R'=>[30,17,17,30,20,18,17],
        b'S'=>[15,16,16,14,1,1,30], b'T'=>[31,4,4,4,4,4,4],
        b'U'=>[17,17,17,17,17,17,14], b'V'=>[17,17,17,17,17,10,4],
        b'W'=>[17,17,17,21,21,21,10], b'X'=>[17,17,10,4,10,17,17],
        b'Y'=>[17,17,10,4,4,4,4], b'Z'=>[31,1,2,4,8,16,31],
        b'0'=>[14,17,19,21,25,17,14], b'1'=>[4,12,4,4,4,4,14],
        b'2'=>[14,17,1,2,4,8,31], b'3'=>[30,1,1,14,1,1,30],
        b'4'=>[2,6,10,18,31,2,2], b'5'=>[31,16,16,30,1,1,30],
        b'6'=>[14,16,16,30,17,17,14], b'7'=>[31,1,2,4,8,8,8],
        b'8'=>[14,17,17,14,17,17,14], b'9'=>[14,17,17,15,1,1,14],
        b':'=>[0,4,4,0,4,4,0], b'.'=>[0,0,0,0,0,12,12],
        b'-'=>[0,0,0,31,0,0,0], b'/'=>[1,2,2,4,8,8,16],
        b'_'=>[0,0,0,0,0,0,31], b'+'=>[0,4,4,31,4,4,0],
        b'>'=>[16,8,4,2,4,8,16], b' '=>[0;7],
        _=>[31,17,5,4,0,4,0],
    }
}

unsafe fn text(fb: &Framebuffer, x: usize, y: usize, s: &[u8], scale: usize, c: u32) {
    let scale = scale.max(1);
    for (i, &ch) in s.iter().enumerate() {
        let bits = glyph(ch);
        for (row, mask) in bits.iter().enumerate() {
            for col in 0..5 {
                if mask & (1 << (4-col)) != 0 {
                    unsafe {
                        rect(fb, x+i*6*scale+col*scale, y+row*scale, scale, scale, c);
                    }
                }
            }
        }
    }
}

unsafe fn draw_splash(fb: &Framebuffer, message: &[u8], progress: usize) {
    let w=fb.width as usize; let h=fb.height as usize;
    let scale=(w/800).clamp(1,2); let cx=w/2; let cy=h/2;
    let bar_w=w.min(360*scale); let bar_x=cx.saturating_sub(bar_w/2);
    unsafe {
        fill(fb,color(fb,DARK));
        rect(fb,cx.saturating_sub(27*scale),cy.saturating_sub(105*scale),54*scale,54*scale,color(fb,CYAN));
        text(fb,cx.saturating_sub(9*scale),cy.saturating_sub(88*scale),b"E",4*scale,color(fb,DARK));
        text(fb,cx.saturating_sub(90*scale),cy.saturating_sub(31*scale),b"EDORA OS",4*scale,color(fb,WHITE));
        text(fb,cx.saturating_sub(60*scale),cy.saturating_sub(4*scale),b"RUST V1.0",2*scale,color(fb,CYAN));
        text(fb,bar_x,cy+42*scale,message,scale,color(fb,MUTED));
        rect(fb,bar_x,cy+62*scale,bar_w,8*scale,color(fb,PANEL2));
        rect(fb,bar_x,cy+62*scale,bar_w.saturating_mul(progress.min(100))/100,8*scale,color(fb,CYAN));
        text(fb,bar_x,cy+80*scale,b"EDORA SYSTEM STARTUP",scale,color(fb,MUTED));
    }
}

unsafe fn draw_desktop(fb: &Framebuffer) {
    let w=fb.width as usize; let h=fb.height as usize;
    let u=(w/1000).clamp(1,2); let m=20*u; let top_h=52*u; let bottom_h=34*u;
    let side_w=(170*u).min(w/3); let gap=16*u;
    let content_x=side_w+m+gap; let content_w=w.saturating_sub(content_x+m);
    let content_y=top_h+m; let content_h=h.saturating_sub(content_y+bottom_h+m);
    unsafe {
        fill(fb,color(fb,BG));
        rect(fb,0,0,w,top_h,color(fb,PANEL)); rect(fb,0,top_h-2*u,w,2*u,color(fb,CYAN));
        rect(fb,m,15*u,23*u,23*u,color(fb,CYAN));
        text(fb,m+5*u,22*u,b"E",u,color(fb,BG));
        text(fb,m+32*u,18*u,b"EDORA OS RUST V1.0",2*u,color(fb,WHITE));
        text(fb,m+32*u,34*u,b"SYSTEM DESKTOP",u,color(fb,MUTED));
        let sx=w.saturating_sub(130*u); rect(fb,sx,21*u,8*u,8*u,color(fb,GREEN));
        text(fb,sx+14*u,20*u,b"ONLINE",u,color(fb,GREEN));
        rect(fb,m,content_y,side_w,content_h,color(fb,PANEL));
        text(fb,m+14*u,content_y+18*u,b"WORKSPACE",u,color(fb,MUTED));
        let ny=content_y+42*u;
        text(fb,m+20*u,ny+11*u,b"01 HOME",u,color(fb,MUTED));
        text(fb,m+20*u,ny+54*u,b"02 SYSTEM",u,color(fb,MUTED));
        text(fb,m+20*u,ny+86*u,b"03 MEMORY",u,color(fb,MUTED));
        text(fb,m+20*u,ny+118*u,b"04 TERMINAL",u,color(fb,MUTED));
        text(fb,content_x,content_y+4*u,b"OVERVIEW",3*u,color(fb,WHITE));
        text(fb,content_x,content_y+30*u,b"YOUR SYSTEM AT A GLANCE",u,color(fb,MUTED));
        let card_y=content_y+58*u; let card_h=104*u; let cg=12*u;
        let card_w=content_w.saturating_sub(cg*2)/3; let x1=content_x;
        let x2=x1+card_w+cg; let x3=x2+card_w+cg;
        rect(fb,x1,card_y,card_w,card_h,color(fb,PANEL)); rect(fb,x1,card_y,card_w,3*u,color(fb,CYAN));
        text(fb,x1+12*u,card_y+14*u,b"KERNEL",u,color(fb,MUTED));
        text(fb,x1+12*u,card_y+39*u,b"RUST",3*u,color(fb,CYAN));
        text(fb,x1+12*u,card_y+78*u,b"BARE METAL",u,color(fb,WHITE));
        rect(fb,x2,card_y,card_w,card_h,color(fb,PANEL)); rect(fb,x2,card_y,card_w,3*u,color(fb,PURPLE));
        text(fb,x2+12*u,card_y+14*u,b"ARCH",u,color(fb,MUTED));
        text(fb,x2+12*u,card_y+39*u,b"X86_64",3*u,color(fb,PURPLE));
        text(fb,x2+12*u,card_y+78*u,b"LIMINE",u,color(fb,WHITE));
        rect(fb,x3,card_y,card_w,card_h,color(fb,PANEL)); rect(fb,x3,card_y,card_w,3*u,color(fb,GREEN));
        text(fb,x3+12*u,card_y+14*u,b"DISPLAY",u,color(fb,MUTED));
        text(fb,x3+12*u,card_y+39*u,b"32-BIT",3*u,color(fb,GREEN));
        text(fb,x3+12*u,card_y+78*u,b"FRAMEBUFFER",u,color(fb,WHITE));
        let py=card_y+card_h+gap; let ph=content_y.saturating_add(content_h).saturating_sub(py);
        rect(fb,content_x,py,content_w,ph,color(fb,PANEL));
        text(fb,content_x+14*u,py+15*u,b"SYSTEM STATUS",u,color(fb,CYAN));
        rect(fb,content_x+14*u,py+37*u,content_w.saturating_sub(28*u),u,color(fb,PANEL2));
        let ry=py+54*u; let col2=content_x+content_w/2;
        text(fb,content_x+14*u,ry,b"BOOT PROTOCOL",u,color(fb,MUTED));
        text(fb,content_x+14*u,ry+20*u,b"LIMINE",2*u,color(fb,WHITE));
        text(fb,col2,ry,b"MEMORY MAP",u,color(fb,MUTED));
        text(fb,col2,ry+20*u,b"REQUESTED",2*u,color(fb,GREEN));
        text(fb,content_x+14*u,ry+58*u,b"GRAPHICS",u,color(fb,MUTED));
        text(fb,content_x+14*u,ry+78*u,b"FRAMEBUFFER ACTIVE",u,color(fb,CYAN));
        text(fb,col2,ry+58*u,b"STATUS",u,color(fb,MUTED));
        text(fb,col2,ry+78*u,b"RUNNING",2*u,color(fb,GREEN));
        rect(fb,0,h.saturating_sub(bottom_h),w,bottom_h,color(fb,PANEL));
        rect(fb,0,h.saturating_sub(bottom_h),w,u,color(fb,PANEL2));
        text(fb,m,h.saturating_sub(bottom_h)+12*u,b"EDORA OS RUST V1.0",u,color(fb,CYAN));
        let right=w.saturating_sub(100*u);
        rect(fb,right,h.saturating_sub(bottom_h)+12*u,7*u,7*u,color(fb,GREEN));
        text(fb,right+13*u,h.saturating_sub(bottom_h)+10*u,b"ONLINE",u,color(fb,WHITE));
    }
}

fn read_key() -> Option<u8> {
    unsafe {
        // Drain a small number of pending bytes, ignoring auxiliary (mouse) data.
        for _ in 0..8 {
            let status = inb(0x64);
            if status & 1 == 0 { return None; }
            let data = inb(0x60);
            if status & 0x20 != 0 { continue; }
            if data == 0xE0 || data == 0xE1 || data & 0x80 != 0 { continue; }
            return Some(data);
        }
    }
    None
}

unsafe fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!("in al, dx", in("dx") port, out("al") value, options(nomem, nostack, preserves_flags));
    }
    value
}

unsafe fn render(fb: &Framebuffer, selected: usize, page: Option<usize>) {
    unsafe {
        draw_desktop(fb);
        let w=fb.width as usize; let h=fb.height as usize;
        let u=(w/1000).clamp(1,2); let m=20*u; let top_h=52*u; let bottom_h=34*u;
        let side_w=(170*u).min(w/3); let content_x=side_w+m+16*u;
        let content_y=top_h+m; let content_w=w.saturating_sub(content_x+m);
        let content_h=h.saturating_sub(content_y+bottom_h+m);
        let ny=content_y+42*u;
        let offsets=[0usize,43,75,107];
        let labels:[&[u8];4]=[b"01 HOME",b"02 SYSTEM",b"03 MEMORY",b"04 TERMINAL"];
        for i in 0..4 {
            let row_y=ny+offsets[i]*u;
            rect(fb,m+7*u,row_y,side_w.saturating_sub(14*u),34*u,
                color(fb,if i==selected {PANEL2} else {PANEL}));
            if i==selected { rect(fb,m+7*u,row_y,3*u,34*u,color(fb,CYAN)); }
            text(fb,m+20*u,row_y+11*u,labels[i],u,
                color(fb,if i==selected {CYAN} else {MUTED}));
        }

        if let Some(p) = page {
            let py=content_y;
            rect(fb,content_x,py,content_w,content_h,color(fb,PANEL));
            let title:[&[u8];4]=[b"HOME",b"SYSTEM INFORMATION",b"MEMORY MANAGER",b"TERMINAL"];
            text(fb,content_x+18*u,py+18*u,title[p.min(3)],2*u,color(fb,CYAN));
            rect(fb,content_x+18*u,py+45*u,content_w.saturating_sub(36*u),u,color(fb,PANEL2));
            match p {
                1 => {
                    text(fb,content_x+18*u,py+65*u,b"EDORA OS RUST V1.0",u,color(fb,WHITE));
                    text(fb,content_x+18*u,py+90*u,b"ARCH: X86_64",u,color(fb,WHITE));
                    text(fb,content_x+18*u,py+115*u,b"BOOTLOADER: LIMINE",u,color(fb,WHITE));
                    text(fb,content_x+18*u,py+140*u,b"GRAPHICS: FRAMEBUFFER",u,color(fb,GREEN));
                }
                2 => {
                    text(fb,content_x+18*u,py+65*u,b"MEMORY MAP REQUEST SENT",u,color(fb,WHITE));
                    text(fb,content_x+18*u,py+90*u,b"MEMORY INIT WAS CALLED",u,color(fb,WHITE));
                    text(fb,content_x+18*u,py+115*u,b"DETAILED STATS NOT WIRED",u,color(fb,MUTED));
                }
                3 => {
                    text(fb,content_x+18*u,py+65*u,b"TERMINAL PREVIEW",u,color(fb,WHITE));
                    text(fb,content_x+18*u,py+90*u,b"INTERACTIVE SHELL",u,color(fb,MUTED));
                    text(fb,content_x+18*u,py+110*u,b"NOT IMPLEMENTED YET",u,color(fb,RED));
                }
                _ => {
                    text(fb,content_x+18*u,py+65*u,b"WELCOME TO EDORA OS",u,color(fb,WHITE));
                    text(fb,content_x+18*u,py+90*u,b"USE UP DOWN AND ENTER",u,color(fb,MUTED));
                }
            }
            text(fb,content_x+18*u,py+content_h.saturating_sub(30*u),b"H OR ESC: HOME",u,color(fb,MUTED));
        } else {
            text(fb,content_x,content_y+4*u,b"OVERVIEW",3*u,color(fb,WHITE));
        }
    }
}
