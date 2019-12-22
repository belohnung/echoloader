use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::{Cursor, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "echoloader", about = "uploads a file over telnet")]
struct Opt {
    /// Remote Address
    #[structopt(parse(try_from_str))]
    address: SocketAddr,

    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Remote file path, using input file name when not specified
    #[structopt(parse(from_os_str))]
    remote_path: Option<PathBuf>,

    /// Chunk size
    #[structopt(short = "s", long = "chunk-size", default_value = "128")]
    chunk_size: usize,

    /// Send delay
    #[structopt(short = "d", long = "delay")]
    delay: Option<u64>,
}

fn main() {
    let opt = Opt::from_args();

    let mut stream = TcpStream::connect(opt.address).unwrap();
    let payload = fs::read(&opt.input).expect("Spooopy");
    let payload_size = payload.len() as u64;
    let mut payload_cursor = Cursor::new(payload);
    let progress_bar = ProgressBar::new(payload_size);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.green/grey}] {bytes}/{total_bytes}({bytes_per_sec}) ({eta})")
        .progress_chars("#>-"));
    let mut cursor = 0;
    let mut chunk_size = opt.chunk_size as u64;
    let remote_file_path = if let Some(remote_file_path) = &opt.remote_path {
        remote_file_path.display()
    } else {
        opt.input.display()
    };

    if chunk_size > payload_size {
        chunk_size = payload_size;
    }

    while cursor < payload_size as u64 {
        if cursor + chunk_size > payload_size {
            chunk_size -= cursor + chunk_size - payload_size;
        }
        let mut buf = vec![0u8; chunk_size as usize];
        payload_cursor.read_exact(&mut buf).unwrap();
        let mut input = hex::encode(buf);
        hex_escape(&mut input);
        stream.write_all(b"echo -n -e \"").unwrap();
        stream.write_all(input.as_bytes()).unwrap();

        if cursor == 0 {
            stream
                .write_all(format!("\" > {}", &remote_file_path).as_bytes())
                .unwrap();
        } else {
            stream
                .write_all(format!("\" >> {}", &remote_file_path).as_bytes())
                .unwrap();
        }
        stream.write_all(b"\n").unwrap();

        progress_bar.set_position(cursor);
        cursor += chunk_size;
        payload_cursor.set_position(cursor);
        if opt.delay.is_some() {
            sleep(Duration::from_millis(opt.delay.unwrap()));
        }
    }
    println!("{} bytes sent.\n", cursor)
}

fn hex_escape(input: &mut String) {
    let ins = r"\x";
    for i in (0..input.len() * 2).step_by(ins.len() * 2) {
        input.insert_str(i, ins);
    }
}
