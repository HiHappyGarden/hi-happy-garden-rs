#![allow(dead_code)]

use osal_rs::utils::Result;


pub trait Initializable<'a> {
    
    fn init(&'a mut self) -> Result<()>;
    
}

pub trait Deinitializable {
    
    fn deinit(&mut self) -> Result<()>;
    
}