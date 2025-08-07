fn deliver_order(){}
 
mod front_of_house;

pub use crate::front_of_house::hosting;

mod back_of_house {

    pub fn cook_order() {}
    pub struct Breakfast {
        pub toast: String,
        seasonal_fruit: String
    }
    impl Breakfast {
        pub fn summer(toast: &str) -> Breakfast {
            Breakfast {
                toast: String::from(toast),
                seasonal_fruit: String::from("peaches"),
            }
        }
    }
    fn fix_incorrect_order() {
        cook_order();
        // move up one to access deliver_order since it lives in the same parent as boh.

        super::deliver_order()
    }
}






pub fn eat_at_restaurant(){
    
     hosting::add_to_waitinglist();

    // Order a breakfast in the summer with Rye toast.
    let mut meal = back_of_house::Breakfast::summer("Whole wheat");
    meal.toast = String::from("Rye");
    
    println!("I'd like {} toast please", meal.toast);

    //private field wil give error
    // meal.seasonal_fruit = String::from("blueberries");

}



