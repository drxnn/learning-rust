// `Vec<T>` is generic over the type `T`. In most cases, the compiler is able to
// infer `T`, for example after pushing a value with a concrete type to the vector.
// But in this exercise, the compiler needs some help through a type annotation.

fn main() {
    // TODO: Fix the compiler error by annotating the type of the vector
    // `Vec<T>`. Choose `T` as some integer type that can be created from
    // `u8` and `i8`.
    #[derive(Debug)]
    enum Numbers {
        I8(i8),
        U8(u8),
    }

    let mut numbers: Vec<Numbers> = Vec::new();

    // Don't change the lines below.
    let n1: u8 = 42;
    numbers.push(Numbers::U8(n1));
    let n2: i8 = -1;
    numbers.push(Numbers::I8(n2));

    println!("{numbers:?}");
}
