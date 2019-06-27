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
        //Chip select
        if let Err(_) = self.cs.set_low() {
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
        if let Err(_) = self.cs.set_high() {
            return Err(());
        };
        if failure {
            return Err(());
        };
        Ok(())
    }

    /// Refresh function. Should be called periodically with >1Hz to update display
    pub fn refresh(&mut self) -> Result<(), ()> {
        
        //Chipselect
        if let Err(_) = self.cs.set_low() {
            return Err(());
        };
        
        let cmd = self.command(CommandBit::WriteCmd);
        let mut failure = false;
        if let Ok(_) = self.write_byte(cmd) {
            let mut old_line: u8 = 1;
            let mut current_line: u8;
            for i in 0..BUFFER_SIZE {
                if let Err(_) = self.write_byte(self.buffer[i]) {
                    failure = true;
                    break;
                }
                current_line = (((i+1)/(WIDTH as usize/8)) + 1) as u8;
                if current_line != old_line {
                    if let Err(_) = self.write_byte(0) {
                        failure = true;
                        break;
                    }
                    if current_line <= HEIGHT {
                        if let Err(_) = self.write_byte(current_line as u8 ) {
                            failure = true;
                            break;
                        }
                    }
                    old_line = current_line;
                }
            }

            if let Err(_) = self.write_byte(0) {
                failure = true;
            }
        }
        if let Err(_) = self.cs.set_high() {
            return Err(());
        };

        if failure {
            return Err(());
        }
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
            command |= 0x40; //Magic number from datasheet
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

enum CommandBit {
    WriteCmd = 0x80,
    ClearBit = 0x20,
}

fn get_index(x: u8, y: u8) -> (usize, u8) {
    let into = y * WIDTH + x;
    let index = into / 8;
    let bit = into % 8;
    (index as usize, bit)
}
