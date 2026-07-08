pub fn hello() -> String {
    String::from("Hello from aster-core!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hello_works() {
        assert_eq!(hello(), "Hello from aster-core!");
    }
}
