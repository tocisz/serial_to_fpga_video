# Bouncing ball FPGA test

The idea is to send commands to FPGA to set video memory that is mapped to an LCD screen.
As a result, bouncing ball is visible on the screen.

FPGA is programmed by [this project](https://github.com/tocisz/verilog-vesa-ca/tree/vesa-vled).
UART module is taken from [here](https://opencores.org/projects/uart2bus).

## Future ideas

This whole setup is to learn how to use external RAM.

### For FPGA
1. Add memory range to be used to control other modules.
2. Create a module for swapping video memory to and from RAM.

### For this project
Create animation that can be played back by swapping RAM pages to video memory.

Animation upload can take some time,
but playback should be smooth and synchronized with video refresh.