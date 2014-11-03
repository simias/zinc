#![feature(phase)]
#![crate_type="staticlib"]
#![no_std]
#![feature(globs)]

extern crate core;
extern crate zinc;
#[phase(plugin)] extern crate macro_platformtree;

use core::prelude::*;
use zinc::hal::tiva_c::ssi::{Ssi, SsiId};
use zinc::drivers::chario::CharIO;
use core::num::SignedInt;

platformtree!(
  tiva_c@mcu {
    clock {
      source  = "MOSC";
      xtal    = "X16_0MHz";
      pll     = true;
      div     = 5;
    }

    timer {
      /* The mcu contain both 16/32bit and "wide" 32/64bit timers. */
      timer@w0 {
        /* prescale sysclk to 1Mhz since the wait code expects 1us
         * granularity */
        prescale = 80;
        mode = "periodic";
      }
    }

    gpio {
      a {
        uart_rx@0 {
          direction = "in";
          function  = 1;
        }
        uart_tx@1 {
          direction = "in";
          function  = 1;
        }

        ssiclk@2 {
          direction = "in";
          function  = 2;
        }
        ssitx@5 {
          direction = "in";
          function  = 2;
        }
      }
      f {
        txled@2 { direction = "out"; }
      }
    }

    uart {
      uart@0 {
        mode = "115200,8n1";
      }
    }

  }

  os {
    single_task {
      loop = "run";
      args {
        timer = &timer;
        txled = &txled;
        uart  = &uart;
      }
    }
  }
);

fn bit_val(bit: bool) -> u16 {
  match bit {
    true  => 0b1110,
    false => 0b1000,
  }
}

fn nibble_val(n: u8) -> u16 {
  let mut data = 0;

  for b in range(0u, 4).rev() {

    let bit = (n & (1 << b)) != 0;

    data |= bit_val(bit) << (b * 4);
  }

  data
}

fn tx_nibble(ssi: &Ssi, n: u8) {
  ssi.transmit(nibble_val(n));
}

fn tx_rgb(ssi: &Ssi, r: u8, g: u8, b: u8) {
  // We need to send GGRRBB
  tx_nibble(ssi, g >> 4);
  tx_nibble(ssi, g);
  tx_nibble(ssi, r >> 4);
  tx_nibble(ssi, r);
  tx_nibble(ssi, b >> 4);
  tx_nibble(ssi, b);
}

fn reset(ssi: &Ssi) {
  ssi.transmit(0);
  ssi.transmit(0);
  ssi.transmit(0);
  ssi.transmit(0);
}

fn hex(v: u8) -> char {
  match v {
    0x0...0x9 => (('0' as u8) + v) as char,
    0xa...0xf => (('a' as u8) + v - 0xa) as char,
    _         => 'X',
  }
}

fn send_u32(io: &CharIO, val: u32)
{
  let mut v = val;

  io.putc('0');
  io.putc('x');

  for _ in range(0u, 8) {
    let h = v >> 28;

    io.putc(hex(h as u8));

    v = v << 4;
  }

  io.putc('\r');
  io.putc('\n');
}

fn rgb_from_hsv(h: u8, s: u8, v: u8) -> (u8, u8, u8) {
    let c = ((v as i32) * (s as i32) + 0xff) >> 8;

    // Round to the nearest
    let side_len: i32 = (0x100 + 3) / 6;

    let h = h as i32;

    let x = (c * (0x100 - ((h % (side_len * 2) * 6) - 0x100).abs())) >> 8;

    let c = c as u8;
    let x = x as u8;

    if        h < side_len {
        (c, x, 0)
    } else if h < side_len * 2 {
        (x, c, 0)
    } else if h < side_len * 3 {
        (0, c, x)
    } else if h < side_len * 4 {
        (0, x, c)
    } else if h < side_len * 5 {
        (x, 0, c)
    } else {
        (c, 0, x)
    }
}

fn run(args: &pt::run_args) {
  use zinc::hal::timer::Timer;
  use zinc::hal::pin::Gpio;

  let ssi = Ssi::new(SsiId::Ssi0, 16);

  let mut i = 0u;

  loop {
    reset(&ssi);

    i += 10;

    for j in range(0u, 300) {
      
      let (r, g, b) = rgb_from_hsv((i + (j * 8)) as u8, 0xff, 0xff);

      tx_rgb(&ssi, r, g, b);
    }

    args.timer.wait_ms(50);
    // tx_rgb(&ssi, 0x00, 0x0f, 0x00);
    // args.timer.wait(1);
    // tx_rgb(&ssi, 0x0f, 0x00, 0x00);
    // args.timer.wait(1);
    // //ssi.transmit(0);
  }
}
