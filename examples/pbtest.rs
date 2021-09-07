pub mod echo {
    lucat::include_proto!("echo");
}

fn main() {
    println!("hello, echo");

    let request = echo::EchoRequest {
        data: vec![1, 2, 3],
        tag: vec![1],
        name: Some(290),
    };

    println!("request = {:?}", request);
}