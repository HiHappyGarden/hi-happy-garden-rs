use hhg_drivers::pico::hardware::{self, Hardware};
use hhg_traits::initializable::Initializable;
use osal_rs::utils::Result; 



pub struct AppMain {
    hardware: &'static Hardware
}

impl Initializable for AppMain{
    fn init() -> Result<()> {
        todo!()
    }
}

impl AppMain {
    pub const fn new(hardware: &'static Hardware) -> Self {
        Self { hardware }
    }
}