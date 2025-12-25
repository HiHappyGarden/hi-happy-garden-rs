#![allow(dead_code)]

use osal_rs::utils::Result;


pub trait Initializable {
    
    fn init(&mut self) -> Result<()>;
    
}

pub trait Deinitializable {
    
    fn deinit(&mut self) -> Result<()>;
    
}