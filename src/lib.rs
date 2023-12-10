
mod interface;

use interface::*;

#[cfg(test)]
pub mod testing {
    use super::*;

    #[test]
    fn test_of_test() {
        println!("test launched!");
        assert_eq!(2+2, 4);
        let _x = 69;
    }
    #[test]
    fn database(){
       assert_eq!(create_database("./database.db".as_ptr()), 0);
    }

}

