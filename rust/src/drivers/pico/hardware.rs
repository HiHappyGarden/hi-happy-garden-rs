use crate::traits::initializable::Initializable;


pub struct Hardware {

}

impl Initializable for Hardware {
    fn init() -> osal_rs::utils::Result<()> {
        todo!()
    }
}

impl Hardware {
    pub const fn new() -> Self {
        Self {  }
    }
}