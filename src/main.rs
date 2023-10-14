use std::{net::TcpListener, io::{Read, Write}};

use dll_syringe::{process::OwnedProcess, Syringe};
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

fn main() {
    color_eyre::install().unwrap();
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let listener = TcpListener::bind("127.0.0.1:7331").unwrap();

    let target_process = OwnedProcess::find_first_by_name("DF CONNECTED").expect("DF CONNECTED Process not found");
    let syringe = Syringe::for_process(target_process);
    let dll_path = std::env::current_exe().unwrap().parent().unwrap().join("libdfdecomp.dll");
    let _injected_payload = syringe.inject(dll_path).expect("Failed to inject the DLL");

    info!("Injected.");

    let (mut stream, _) = listener.accept().unwrap();
    // End of initialization of the stream

    info!("Connected.");

    let mut stream_buffer = [0u8; 0xffff];
    while let Ok(n) = stream.read(&mut stream_buffer) {
        std::io::stdout().lock().write_all(&stream_buffer[..n]).expect("Unable to write to stdout");
    }
}
