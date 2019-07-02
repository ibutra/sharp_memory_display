//! A platform agnostic driver to interface with the Sharp Memory Display
//!
//! This driver is built using [`embedded-hal`] traits.
//!
#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

use embedded_hal as hal;

const WIDTH: u8 = 144;
const HEIGHT: u8 = 168;
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
        let result = display.cs.set_low();
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
        //Chip select
        if let Err(_) = self.cs.set_high() {
            return Err(());
        };
        let command = self.command(CommandBit::ClearBit);
        let mut failure = false;
        if let Ok(_) = self.write_byte(command) {
            if let Err(_) = self.write_byte(0) {
                failure = true;
            }
        } else {
            failure = true;
        }
        if let Err(_) = self.cs.set_low() {
            return Err(());
        };
        if failure {
            return Err(());
        };
        Ok(())
    }

    /// Refresh function. Should be called periodically with >1Hz to update display
    pub fn refresh(&mut self) -> Result<(), ()> {
        const SIZE: usize = BUFFER_SIZE + 1 + HEIGHT as usize * 2 + 1; //1 byte command, heigh * 2 byte per line (number, data, end), 1 byte end
        let mut buffer: [u8; SIZE] = [0; SIZE];
        buffer[0] = self.command(CommandBit::WriteCmd);
        const BYTES_PER_LINE: u8 = WIDTH / 8 + 2;
        for i in 0..HEIGHT {
            let buffer_index: usize = (i * BYTES_PER_LINE + 1) as usize;
            buffer[buffer_index] = i + 1;
            let slice_index: usize = (i * WIDTH / 8) as usize;
            let slice = &self.buffer[slice_index..slice_index + WIDTH as usize / 8];
            buffer[buffer_index + 1 .. buffer_index + WIDTH as usize / 8  - 1].copy_from_slice(&slice);
            buffer[buffer_index + WIDTH as usize / 8] = 0;
        }
        
        //Chipselect
        //TODO: better error handling
        let _ = self.cs.set_high(); 
        let _ = self.com.write(&buffer);    
        let _ = self.cs.set_low();
        
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

    fn command(&mut self, command: CommandBit) -> u8 {
        let mut command = command as u8;
        if self.vcom {
            command |= 0x02;
        }
        self.toggle_vcom();
        command
    }

    fn write_byte(&mut self, data: u8) -> Result<(), ()> {
        //First send the command bits
        let result = self.com.write(&[data]);
        if result.is_err() {
            return Err(());
        }
        Ok(())
    }

    fn toggle_vcom(&mut self) {
        self.vcom = !self.vcom;
    }
}

impl<E> From<E> for ()
    where
        E: hal::digital::v2::OutputPin::Error
{
    fn from(err: E) -> Self {
        ()
    }

}

enum CommandBit {
    WriteCmd = 0x01,
    ClearBit = 0x04,
}

fn get_index(x: u8, y: u8) -> (usize, u8) {
    let into = y as usize * WIDTH as usize + x as usize;
    let index = into / 8;
    let bit = into % 8;
    (index, bit as u8)
}
