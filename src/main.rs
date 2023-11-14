use std::io::{self, Read, Write};

use casper_types::{
    bytesrepr::{FromBytes, ToBytes},
    BinaryRequest, BlockHash, BlockHeader, DbId, Digest,
};
use clap::Parser;
use juliet::{
    io::IoCoreBuilder, protocol::ProtocolBuilder, rpc::RpcBuilder, ChannelConfiguration, ChannelId,
};
use rand::rngs::ThreadRng;
use tokio::net::TcpStream;

#[derive(Parser, Debug)]
struct Args {
    /// Block hash to retrieve header for
    #[arg(short, long)]
    block_hash: Option<String>,
}

fn main_classic() {
    let args = Args::parse();

    let mut stream = std::net::TcpStream::connect("127.0.0.1:34567").unwrap();
    stream.set_nonblocking(true).unwrap();
    println!("Connected to casper-binary-server");

    let _rng = ThreadRng::default();

    let key = match args.block_hash {
        Some(block_hash) => {
            let digest: Digest = Digest::from_hex(block_hash).unwrap();
            let block_hash: BlockHash = BlockHash::new(digest);
            block_hash.to_bytes().unwrap()
        }
        None => todo!(),
    };

    let req = BinaryRequest::Get {
        db: DbId::BlockHeaderV2,
        key,
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

//--------------------------------------------------------------------------------- JULIET ---------------------------------------------------------------------------------
//--------------------------------------------------------------------------------- JULIET ---------------------------------------------------------------------------------
//--------------------------------------------------------------------------------- JULIET ---------------------------------------------------------------------------------
//--------------------------------------------------------------------------------- JULIET ---------------------------------------------------------------------------------
//--------------------------------------------------------------------------------- JULIET ---------------------------------------------------------------------------------
//--------------------------------------------------------------------------------- JULIET ---------------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let protocol_builder = ProtocolBuilder::<1>::with_default_channel_config(
        ChannelConfiguration::default()
            .with_request_limit(3)
            .with_max_request_payload_size(4096)
            .with_max_response_payload_size(4096),
    );
    let io_builder = IoCoreBuilder::new(protocol_builder).buffer_size(ChannelId::new(0), 16);
    let rpc_builder = Box::leak(Box::new(RpcBuilder::new(io_builder))); // TODO[RC]: Leak?

    let remote_server = TcpStream::connect("127.0.0.1:34568")
        .await
        .expect("failed to connect to server");
    println!("connected to server 127.0.0.1:34568");

    let (reader, writer) = remote_server.into_split();
    let (client, mut server) = rpc_builder.build(reader, writer);

    // We are not using the server functionality, but still need to run it for IO reasons.
    tokio::spawn(async move {
        if let Err(err) = server.next_request().await {
            println!("server read error: {}", err);
        }
    });

    println!("Sending request...");
    let key = match args.block_hash {
        Some(block_hash) => {
            println!("for BlockHash");' 
            let digest: Digest = Digest::from_hex(block_hash).unwrap();
            let block_hash: BlockHash = BlockHash::new(digest);
            block_hash.to_bytes().unwrap()
        }
        None => todo!(),
    };

    let req = BinaryRequest::Get {
        db: DbId::BlockHeaderV2,
        key,
    };
    let payload = req.to_bytes().unwrap();

    dbg!(&payload);

    let request_guard = client
        .create_request(ChannelId::new(0))
        .with_payload(payload.into())
        .queue_for_sending()
        .await;

    println!("sent request");
    match request_guard.wait_for_response().await {
        Ok(response) => {
            // let decoded = String::from_utf8(response.expect("should have payload").to_vec())
            //     .expect("did not expect invalid UTF8");
            let response_payload = response.unwrap();
            dbg!(&response_payload);
            let (block_header, _): (BlockHeader, _) =
                BlockHeader::from_bytes(&response_payload).unwrap();
            println!("{block_header:?}");
        }
        Err(err) => {
            println!("server error: {}", err);
        }
    }
}
