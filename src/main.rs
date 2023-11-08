use std::{
    io::{self, BufRead, Read, Write},
    net::TcpStream,
};

use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    BinaryRequest, BlockHash, BlockHeader, BlockHeaderV2, Digest,
};
use clap::Parser;
use rand::rngs::ThreadRng;

#[derive(Parser, Debug)]
struct Args {
    /// Block hash to retrieve header for
    #[arg(short, long)]
    block_hash: String,
}

fn main() {
    let args = Args::parse();

    let mut stream = TcpStream::connect("127.0.0.1:34567").unwrap();
    stream.set_nonblocking(true).unwrap();
    println!("Connected to casper-binary-server");

    let _rng = ThreadRng::default();

    let digest: Digest = Digest::from_hex(args.block_hash).unwrap();
    let block_hash: BlockHash = BlockHash::new(digest);
    let req = BinaryRequest::Get {
        db: "block_header_v2".to_string(),
        key: block_hash.to_bytes().unwrap(),
    };
    stream
        .write_all(req.to_bytes().unwrap().as_slice())
        .unwrap();
    println!("Message sent, awaiting response...");

    let mut read_buf = Vec::with_capacity(8192);

    loop {
        match stream.read_to_end(&mut read_buf) {
            Ok(_) => break,
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => panic!("encountered IO error: {e}"),
        };
    }

    println!("Got full response of length: {}", read_buf.len());
    if read_buf[0] == 1 {
        println!("Got error response");
        return;
    }

    println!("{}", hex::encode(&read_buf[1..]));

    let (block_header, _): (BlockHeader, _) = BlockHeader::from_bytes(&read_buf[1..]).unwrap();
    println!("{block_header:?}");

    // Try parse to get response
    // let (response, remainder): (BinaryResponse, &[u8]) =
    //     BinaryResponse::from_bytes(buf.as_slice()).unwrap();
    // println!(
    //     "Have BinaryResponse and {} bytes is left in the buffer:\n{:?}",
    //     remainder.len(),
    //     response
    // );
}
