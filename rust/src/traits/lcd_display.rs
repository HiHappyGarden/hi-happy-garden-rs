


/// Trait for LCD display operations
pub trait LCDDisplay {
    /// Draw the buffer to the display
    fn draw(&mut self) -> Result<()>;
    
    /// Clear the display buffer
    fn clear(&mut self);
    
    /// Draw a single pixel at the specified position
    fn draw_pixel(&mut self, x: u8, y: u8, write_mode: LCDSH1106WriteMode) -> Result<()>;
    
    /// Draw a bitmap image
    fn draw_bitmap_image(&mut self, x: u8, y: u8, width: u8, height: u8, image: &[u8], write_mode: LCDSH1106WriteMode) -> Result<()>;
    
    /// Draw a rectangle
    fn draw_rect(&mut self, x: u8, y: u8, width: u8, height: u8, write_mode: LCDSH1106WriteMode) -> Result<()>;
    
    /// Draw a single character
    fn draw_char(&mut self, c: char, x: u8, y: u8, font: &[u8]) -> Result<()>;
    
    /// Draw a string
    fn draw_str(&mut self, str: &str, x: u8, y: u8, font: &[u8], font_size: u32) -> Result<()>;
    
    /// Invert the display orientation
    fn invert_orientation(&mut self) -> Result<()>;
    
    /// Set the display contrast
    fn set_contrast(&self, contrast: u8) -> Result<()>;
    
    /// Turn off the display
    fn turn_off(&mut self) -> Result<()>;
    
    /// Turn on the display
    fn turn_on(&mut self) -> Result<()>;
}
