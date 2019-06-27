# Sharp Memory Display

This is a driver for the sharp memory display sold by [adafruit](https://www.adafruit.com/product/3502).

# Graphics Library

Currently a graphics library is planned which wil be supported by this crate.


## Errortypes
To keep the size to a minimum the Error type of the SPI bus and the IO Pins are not forwarded.
Putting them in a generic Error type increased the size by over 1000 Byte.
Exposing the errors will be explored in the future.
