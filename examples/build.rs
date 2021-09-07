extern crate lucat_build;

fn main() {
    let res = lucat_build::configure()
        .build_client(true)
        .build_server(true)
        .compile(
            &["proto/echo/echo.proto"],
            &["proto/echo"],
        );
    match res {
        Ok(_res) => println!("{}", "OK"),
        Err(_err) => println!("{}", "Err"),
    };
}
