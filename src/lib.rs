
mod init;
mod debug;

use init::*;

#[cfg(test)]
pub mod testing {
    use super::*;

    #[test]
    fn main() {
        println!("test launched!");
        assert_eq!(2+2, 4);
        assert_eq!(X, 69);
        create_database().expect("How in the fucking world did this happen man");
    }   
}

