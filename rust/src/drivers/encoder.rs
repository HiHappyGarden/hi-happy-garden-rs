use osal_rs::utils::Result;

use crate::traits::state::Initializable;

pub struct Encoder;


impl Encoder {
    pub fn new() -> Self {
        Encoder {}
    }
}

impl Initializable for Encoder {

    fn init(&mut self) -> Result<()> {
        
        Ok(())
    }
}