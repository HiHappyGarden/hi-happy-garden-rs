use osal_rs::utils::Result;


pub trait Initializable {
    
    fn init() -> Result<()>;
    
}