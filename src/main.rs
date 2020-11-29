use std::io;
use std::time::Duration;

use serial::prelude::*;

const PORT: &str = "COM6";

// 144 = 16 * 9    -- only x != 0 (mod 9)
// 126 = 0x3f0 * 2 -- only y != 0 (mod 2)
const WIDTH: usize = 144;
const HEIGHT: usize = 126;

struct Image([bool; WIDTH*HEIGHT]);

impl Image {
    pub fn new() -> Image {
        Image([false; WIDTH*HEIGHT])
    }

    pub fn draw_circle(self: &mut Self, cx: i32, cy: i32, d: i32) {
        for y in 0 .. HEIGHT as i32 {
            for x in 0 .. WIDTH as i32 {
                let dx = x - cx;
                let dy = y - cy;
                if dx*dx + dy*dy < d*d {
                    self.0[(y as usize)*WIDTH + (x as usize)] = true;
                }
            }
        }
    }
}

const FRAME_LEN: usize = 0x3f0;
struct Frame([u8;FRAME_LEN]);

impl Frame {
    pub fn new() -> Frame {
        Frame([0; FRAME_LEN])
    }
}

impl From<Image> for Frame {
    fn from(img: Image) -> Self {
        let mut frame = Frame::new();
        let enc = &mut frame.0;
        let buf = &img.0;
        for j in 0 .. 63 {
            for i in 0 .. 16 {
                let x = 9*i;
                let y = 2*j + 1;

                let mut v = 0;
                for idx in WIDTH*y+x .. WIDTH*y+x+8 {
                    v <<= 1;
                    if buf[idx] {
                        v |= 1;
                    }
                }
                enc[16*j+i] = v;
            }
        }
        frame
    }
}

#[allow(dead_code)]
fn write<T: SerialPort>(port: &mut T, addr: u16, val: u8) -> io::Result<()> {
    let s = format!("w{:x} {:x}\n", val, addr);
    let _ = port.write(s.as_bytes())?;
    Ok(())
}

fn bin_write<T: SerialPort>(port: &mut T, addr: u16, data: &[u8]) -> io::Result<()> {
    if data.len() > 256 {
        panic!("Data packet too long!");
    }
    let mut header = [0;5];
    header[0] = 0; // enter binary mode
    header[1] = 0b00100000; // write with auto increment and no ACK
    header[2] = ((addr >> 8) & 0xff) as u8;
    header[3] = (addr & 0xff) as u8;
    header[4] = data.len() as u8; // data length
    let _ = port.write(&header)?;
    let _ = port.write(data)?;
    Ok(())
}

fn fill<T: SerialPort>(port: &mut T, v: u8) -> io::Result<()> {
    let mut addr: usize = 0;
    let full = [v;256];
    while addr < FRAME_LEN {
        let len: usize;
        if FRAME_LEN - addr > 255 {
            len = 255;
        } else {
            len = FRAME_LEN - addr;
        }
        // println!("Write to {} len = {}", addr, len);
        bin_write(port, addr as u16, &full[0..len])?;
        addr += len;
    }
    Ok(())
}

fn send<T: SerialPort>(port: &mut T, img: Image) -> io::Result<()> {
    let mut addr: usize = 0;
    let frame = Frame::from(img).0;
    while addr < frame.len() {
        let len: usize;
        if frame.len() - addr > 255 {
            len = 255;
        } else {
            len = frame.len() - addr;
        }
        // println!("Write to {} len = {}", addr, len);
        bin_write(port, addr as u16, &frame[addr..(addr+len)])?;
        addr += len;
    }
    Ok(())
}

fn main() {
    let mut port = serial::open(PORT).unwrap();
    interact(&mut port).unwrap();
}

fn interact<T: SerialPort>(port: &mut T) -> io::Result<()> {
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud115200)?;
        settings.set_char_size(serial::Bits8);
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_flow_control(serial::FlowNone);
        Ok(())
    })?;
    port.set_timeout(Duration::from_millis(1000))?;

    fill(port, 0)?;

    let r = (HEIGHT/10) as i32;
    let mut cx = (WIDTH/2) as i32;
    let mut cy = (HEIGHT/2) as i32;
    let mut dx = 1;
    let mut dy = 1;
    loop {
        let mut img = Image::new();
        img.draw_circle(cx, cy, r);
        send(port, img)?;

        cx += dx;
        cy += dy;

        if cx >= (WIDTH as i32) - r {
            dx = -1;
        } else if cx <= r {
            dx = 1;
        }
        if cy >= (HEIGHT as i32) - r {
            dy = -1;
        } else if cy <= r {
            dy = 1;
        }
    }

    // Ok(())
}