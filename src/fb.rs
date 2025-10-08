use core::cell::SyncUnsafeCell;

use limine::framebuffer::{Framebuffer, MemoryModel};
use noto_sans_mono_bitmap::{RasterizedChar, get_raster};

use crate::{
    fb::font_data::{
        BACKUP_CHAR, BORDER_PADDING, CHAR_RASTER_HEIGHT, LETTER_SPACING, LINE_SPACING,
    },
    requests::FRAMEBUFFER,
};

pub static PRIMITIVE_WRITER: SyncUnsafeCell<Option<PrimitiveFbWriter>> = SyncUnsafeCell::new(None);

pub fn init_writer() {
    if let Some(framebuffer_response) = FRAMEBUFFER.get_response()
        && let Some(framebuffer) = framebuffer_response.framebuffers().next()
    {
        let writer = PrimitiveFbWriter::new(framebuffer);

        unsafe { *PRIMITIVE_WRITER.get() = Some(writer) }
    }
}

pub fn get_fb_writer() -> &'static mut PrimitiveFbWriter {
    unsafe { PRIMITIVE_WRITER.get().as_mut().unwrap().as_mut().unwrap() }
}

mod font_data {
    use noto_sans_mono_bitmap::{FontWeight, RasterHeight, get_raster_width};

    pub const LINE_SPACING: usize = 2;

    pub const LETTER_SPACING: usize = 0;

    pub const BORDER_PADDING: usize = 1;

    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;

    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);

    pub const BACKUP_CHAR: char = '�';

    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;
}

fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(c, font_data::FONT_WEIGHT, font_data::CHAR_RASTER_HEIGHT)
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}

pub struct PrimitiveFbWriter {
    pub fb: Framebuffer<'static>,
    pub x: usize,
    pub y: usize,
}

impl PrimitiveFbWriter {
    pub fn new(fb: Framebuffer<'static>) -> Self {
        Self { fb, x: 0, y: 0 }
    }
    pub fn width(&self) -> u64 {
        self.fb.width()
    }
    pub fn height(&self) -> u64 {
        self.fb.height()
    }
    pub fn write_pixel(&mut self, x: u64, y: u64, color: u32) {
        let pixel_offset = y * self.fb.pitch() + x * 4;
        let offset =
            usize::try_from(pixel_offset).expect("Cannot convert the pixel offset to usize");
        unsafe {
            #[allow(clippy::cast_ptr_alignment)]
            let buffer = self.fb.addr().add(offset).cast::<u32>();

            *buffer = color;
        }
    }
    fn write_rendered_char(&mut self, rendered_char: &RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                let pixel_x = (self.x + x) as u64;
                let pixel_y = (self.y + y) as u64;
                let intensity = u32::from(*byte);
                let color = (intensity << 16) | (intensity << 8) | intensity;

                self.write_pixel(pixel_x, pixel_y, color);
            }
        }
        self.x += rendered_char.width() + LETTER_SPACING;
    }
    pub fn newline(&mut self) {
        self.y += CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return();
    }
    pub fn carriage_return(&mut self) {
        self.x = BORDER_PADDING;
    }
    pub fn clear(&mut self) {
        self.x = 0;
        self.y = 0;

        unsafe {
            core::ptr::write_bytes(self.fb.addr(), 0x0, (self.width() * self.height()) as usize);
        }
    }
}

impl core::fmt::Write for PrimitiveFbWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }

        Ok(())
    }
    fn write_char(&mut self, c: char) -> core::fmt::Result {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                let new_xpos = self.x + font_data::CHAR_RASTER_WIDTH;

                if new_xpos >= self.width() as usize {
                    self.newline();
                }
                let new_ypos = self.y + font_data::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.height() as usize {
                    self.clear();
                }
                self.write_rendered_char(&get_char_raster(c));
            }
        };

        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg: tt)*) => {
        let fb = $crate::fb::get_fb_writer();

        fb.write_fmt(format_args!($($arg)*)).unwrap();
    }
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::print!($($arg)*);
        $crate::print!("\n");
    };
    () => {
        $crate::print!("\n");
    };
}
