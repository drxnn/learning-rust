pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

// helper functions
fn larger() -> Rectangle {
    Rectangle { width: 100, height: 100 }
}

fn smaller() -> Rectangle {
    Rectangle { width: 1, height: 1 }
}

pub fn greeting(name: &str) -> String {
    format!("Hello {name}!")
}

#[cfg(test)]
mod tests {
    use super::*;

    
    #[test]
    fn larger_can_hold_smaller_test(){
    

        assert!(larger().can_hold(&smaller()));
    }

    #[test]
    fn smaller_cannot_hold_larger() {
        

        assert!(!smaller().can_hold(&larger()));
    }

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }


    #[test]
    fn test_that_fails() {
        panic!("FAILED")
    }

    #[test]
    fn greeting_contains_name() {
        let result = greeting("Carol");
        assert!(result.contains("Carol"),"Greeting did not contain name, value was `{result}`"
        );
    }
}
