fn main() {
    let borrowed;
    {
        let input = String::from("\"text\"");
        let mut scanner = jzon::Scanner::new_str(&input);
        borrowed = scanner.read_str().unwrap().as_borrowed().unwrap();
    }
    println!("{borrowed}");
}
