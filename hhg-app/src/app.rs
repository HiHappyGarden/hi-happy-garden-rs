

#[unsafe(no_mangle)]
pub unsafe extern "C" fn app_main() {

}

#[unsafe(no_mangle)]
pub extern "C" fn app_add(left: u64, right: u64) -> u64 {
    left + right
}