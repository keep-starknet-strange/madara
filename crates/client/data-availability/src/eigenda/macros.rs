#[macro_export]
macro_rules! grpcurl_command {
    ($($arg:expr),*) => {{
        let mut command = std::process::Command::new("grpcurl");
        $(
            command.arg($arg);
        )*
        command.output() 
        // This returns a Result<std::process::Output, std::io::Error>
    }};
}