async fn do_things1() {
    println!("hello, world!")
}

#[tokio::main]
async fn main() {
    do_things1().await;
}
