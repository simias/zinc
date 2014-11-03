// Zinc, the bare metal stack for rust.
// Copyright 2014 Lionel Flandrin <lionel@svkt.org>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! SSI configuration

use hal::tiva_c::sysctl;
use util::support::get_reg_ref;

#[path="../../util/wait_for.rs"] mod wait_for;

/// There are 4 SSI instances in total
#[allow(missing_docs)]
pub enum SsiId {
  Ssi0,
  Ssi1,
  Ssi2,
  Ssi3,
}

/// Structure describing a single SSI module
pub struct Ssi {
  /// SSI register interface
  regs: &'static reg::Ssi,
}

impl Ssi {
  /// Create and setup a SSI
  pub fn new(id: SsiId, data_size: uint) -> Ssi {
    let (periph, regs) = match id {
      SsiId::Ssi0 => (sysctl::periph::ssi::SSI_0, reg::SSI_0),
      SsiId::Ssi1 => (sysctl::periph::ssi::SSI_1, reg::SSI_1),
      SsiId::Ssi2 => (sysctl::periph::ssi::SSI_2, reg::SSI_2),
      SsiId::Ssi3 => (sysctl::periph::ssi::SSI_3, reg::SSI_3),
    };

    let ssi = Ssi { regs: get_reg_ref(regs) };

    periph.ensure_enabled();

    ssi.configure(data_size);

    ssi
  }

  /// SSI register configuration
  pub fn configure(&self, data_size: uint) {
    // Disable SSI before config
    self.regs.cr1.set_sse(false);

    // SSI master mode
    self.regs.cr1.set_ms(reg::Ssi_cr1_ms::Master);

    self.regs.cpsr.set_cpsdvsr(2);

    self.regs.cr0
      .set_scr(14)
      .set_frf(reg::Ssi_cr0_frf::Spi)
      .set_sph(false)
      .set_spo(false)
      .set_dss((data_size - 1) as u32);

    // Enable SSI
    self.regs.cr1.set_sse(true);
  }

  /// Transmit a word on the serial interface
  pub fn transmit(&self, data: u16) {
    wait_for!(self.regs.sr.tnf());

    self.regs.dr.set_data(data as u32);
  }
}

pub mod reg {
  //! SSI registers definition
  use util::volatile_cell::VolatileCell;
  use core::ops::Drop;

  ioregs!(Ssi = {
    0x00 => reg32 cr0 {
      0..3   => dss,     //= Data size select
      4..5   => frf {    //! SSI frame format select
        0 => Spi,
        1 => TiSs,
        2 => Microwire, 
      }
      6      => spo,     //= Clock polarity
      7      => sph,     //= Clock phase
      8..15  => scr,     //= Clock rate
    }
    0x04 => reg32 cr1 {
      0      => lbm,     //= Loopback mode
      1      => sse,     //= Syncronous Serial port Enable
      2      => ms {     //! master/slave select
        0 => Master,
        1 => Slave,
      }
      4      => eot,     //= End Of Transmission
    }
    0x08 => reg32 dr {
      0..15  => data,    //= Rx/Tx Data
    }
    0x0c => reg32 sr {
      0      => tfe,     //= Tx FIFO empty
      1      => tnf,     //= Tx FIFO not full
      2      => rne,     //= Rx FIFO not empty
      3      => rff,     //= Rx FIFO full
      4      => bsy,     //= SSI busy flag
    }
    0x10 => reg32 cpsr {
      0..7   => cpsdvsr, //= Clock prescale divisor
    }
  });

  pub const SSI_0: *const Ssi = 0x40008000 as *const Ssi;
  pub const SSI_1: *const Ssi = 0x40009000 as *const Ssi;
  pub const SSI_2: *const Ssi = 0x4000A000 as *const Ssi;
  pub const SSI_3: *const Ssi = 0x4000B000 as *const Ssi;
}
