
mod init;
mod debug;

use init::init::*;

#[cfg(test)]
pub mod testing {
    #[test]
    fn main() {
//        use x;
        println!("test launched!");
        assert_eq!(2+2, 4);
        assert_eq!(x, 69);
    }   
}

