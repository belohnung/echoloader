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
pub struct Opt {
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

pub fn hex_escape(input: &mut String) {
    let mut output = String::new();
    for c in input.chars() {
        output.push_str(&format!("\\x{:02x}", c as u8));
    }
    *input = output;
}

impl Opt {
    pub fn new(
        address: SocketAddr,
        input: PathBuf,
        remote_path: Option<PathBuf>,
        chunk_size: usize,
        delay: Option<u64>,
    ) -> Self {
        Opt {
            address,
            input,
            remote_path,
            chunk_size,
            delay,
        }
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn input(&self) -> PathBuf {
        self.input.clone()
    }

    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    pub fn delay(&self) -> Option<u64> {
        self.delay
    }

    pub fn remote_path(&self) -> Option<PathBuf> {
        self.remote_path.clone()
    }
}

pub fn main_with_args(opt: Opt) {
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
            if let Err(err) = stream.write_all(format!("\" > {}", &remote_file_path).as_bytes()) {
                eprintln!("Error writing to stream: {}", err);
                return;
            }
        } else if let Err(err) = stream.write_all(format!("\" >> {}", &remote_file_path).as_bytes())
        {
            eprintln!("Error writing to stream: {}", err);
            return;
        } else {
            stream.write_all(b"\n").unwrap();
        }

        progress_bar.set_position(cursor);
        cursor += chunk_size;
        payload_cursor.set_position(cursor);
        if opt.delay.is_some() {
            sleep(Duration::from_millis(opt.delay.unwrap()));
        }
    }
    println!("{} bytes sent.\n", cursor)
}
