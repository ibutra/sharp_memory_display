//! A platform agnostic driver to interface with the Sharp Memory Display
//!
//! This driver is built using [`embedded-hal`] traits.
//!
#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

use embedded_hal as hal;

const WIDTH: u8 = 44;
const HEIGHT: u8 = 68;
const BUFFER_SIZE: usize = WIDTH as usize * HEIGHT as usize / 8; //this works because 144*168 is dividable by 8 if the dimensions ever change make sure to handle this!

/// The Sharp Memory Display driver
pub struct Display<SPI, CS>
where
    SPI: hal::blocking::spi::Write<u8>,
    CS: hal::digital::v2::OutputPin,
{
    com: SPI,
    cs: CS,
    buffer: [u8; BUFFER_SIZE],
    vcom: bool,
}

impl<SPI, CS, SpiError, IoError> Display<SPI, CS>
where
    SPI: hal::blocking::spi::Write<u8, Error = SpiError>,
    CS: hal::digital::v2::OutputPin<Error = IoError>,
{
    ///Creates a new display driver
    pub fn new(spi: SPI, cs: CS) -> Result<Display<SPI, CS>, ()> {
        let mut display = Display {
            com: spi,
            cs: cs,
            buffer: [0; BUFFER_SIZE],
            vcom: true,
        };
        let result = display.cs.set_high();
        if result.is_err() {
            return Err(());
        }
        display.clear()?;
        Ok(display)
    }

    /// Clear the display buffer and therefore the display
    pub fn clear(&mut self)  -> Result<(), ()> {
        for val in self.buffer.iter_mut() {
            *val = 0;
        }
        let buf = [0x00];
        self.write_bytes(CommandBit::ClearBit, &buf)?;
        Ok(())
    }

    /// Refresh function. Should be called periodically with >1Hz to update display
    pub fn refresh(&mut self) -> Result<(), ()> {
        // const SEND_BUFF_SIZE: usize = BUFFER_SIZE + 2 * HEIGHT as usize + 1;
        // let mut buf : [u8; SEND_BUFF_SIZE] = [0; SEND_BUFF_SIZE]; // Size of Buffer plus line number plus end line bit and one final zero bit
        // let byte_per_line = BUFFER_SIZE / HEIGHT as usize; //If this isn't even we have a problem houston
        // for line in 0..HEIGHT {
        //     let index = line as usize * (byte_per_line + 2);
        //     buf[index] = line;
        //     for byte in 0..byte_per_line {
        //         buf[index + byte] = self.buffer[line as usize * byte_per_line + byte];
        //     }
        //     buf[index + byte_per_line] = 0;
        // }
        // buf[BUFFER_SIZE + 2 * HEIGHT as usize + 1] = 0;

        // self.write_bytes(CommandBit::WriteCmd, &buf)?;
        Ok(())

    }

    /// Set a pixel
    /// The pixel are numerated started from the top left starting with 0,0
    pub fn set_pixel(&mut self, x: u8, y: u8, black: bool) {
        if x > WIDTH || y > HEIGHT {
            return;
        }
        let (index, bit) = get_index(x, y);
        if black {
            self.buffer[index] |= 1 << bit;
        } else {
            self.buffer[index] &= !(1 << bit);
        }
    }

    /// Get a pixel
    pub fn get_pixel(&mut self, x: u8, y: u8) -> Option<bool> {
        if x > WIDTH || y > HEIGHT {
            return None;
        }
        let (index, bit) = get_index(x, y);
        Some((self.buffer[index] & 1 << bit) != 0)

    }

    fn write_bytes(&mut self, command: CommandBit, data: &[u8]) -> Result<(), ()> {

        let mut cmd = command as u8;
        if self.vcom {
            cmd |= 0x40; //Magic number from datasheet
        };
        self.toggle_vcom();
        
        //Chip select
        let cs_result = self.cs.set_low();
        if cs_result.is_err() {
            return Err(());
        }
        //First send the command bits
        let mut result = self.com.write(&[cmd]);
        if result.is_ok() { //If command bits were send successfully write the actual data
            result = self.com.write(&data);
        }

        //Disable chip select
        let cs_result = self.cs.set_high();
        if result.is_err() { //Spi Error has precedence over io error
            return Err(());
        }
        if cs_result.is_err() {
            return Err(());
        }
        Ok(())
    }

    fn toggle_vcom(&mut self) {
        self.vcom = !self.vcom;
    }
}

enum CommandBit {
    // WriteCmd = 0x80,
    ClearBit = 0x20,
}

fn get_index(x: u8, y: u8) -> (usize, u8) {
    let into = y * WIDTH + x;
    let index = into / 8;
    let bit = into % 8;
    (index as usize, bit)
}
