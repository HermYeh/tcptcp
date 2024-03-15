
extern crate encoding;
extern crate console;

use encoding::{Encoding, EncoderTrap};
use encoding::all::ASCII;
use std::error;
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::{Read, Write, Result};
use std::str;
use std::fs::File;
use std::fs;

use console::{Term, style};

/*
How my protocol works:
- Both client and server communicate using an 8 byte buffer
- Upon connection, client will attempt to send a message
- Client calculates message size, sends size to server
- Server catches message size and loops in order to assemble message
*/

fn encode_message_size(cmd: &str) -> Result<Vec<u8>> {
    let mut message_size = cmd.len();
    //println!("{:?}", cmd);
    message_size = message_size + 1;
    let message_size_str = message_size.to_string();
    let mut message_size_bytes = ASCII.encode(&message_size_str, EncoderTrap::Strict).map_err(|x| x.into_owned()).unwrap();
    message_size_bytes.push('\r' as u8);
    
    //Ok(String::from_utf8(string_size_bytes).unwrap())
    Ok(message_size_bytes)
}

fn encode_message(cmd: &str) -> Result <Vec<u8>> {
    //println!("{:?}", cmd);
    let message_str = cmd.to_string();
    let mut message_bytes = ASCII.encode(&message_str, EncoderTrap::Strict).map_err(|x| x.into_owned()).unwrap();
    message_bytes.push('\r' as u8);

    //Ok(String::from_utf8(string_size_bytes).unwrap())
    Ok(message_bytes)
}

fn check_ack(mut ack_buf: &mut [u8]) -> String {

    let ack_slice: &str = str::from_utf8(&mut ack_buf).unwrap(); //string slice
    let mut ack_str = ack_slice.to_string(); //convert slice to string
    let index: usize = ack_str.rfind('\r').unwrap();
    //println!("{:?} server ACK:", ack_str.split_off(index));
    format!("{:?}", ack_str.split_off(index)); 
    if ack_str != "ACK"{
        //println!("received ACK from server");
        // end with error, maybe set a timeout
        return String::from("error")
    }
    String::from("ACK")
}

pub fn filetransfer(incomingtype: String, listener: TcpListener) {
    if incomingtype == "file" {
        println!("listenering for file...");
        match file(listener) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
            }
        };
    } else if incomingtype == "wechat" {
        match wechat(listener) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e);
            }
        };
    };
}
pub fn handle_wechat(mut stream: TcpStream) -> Result<(), std::io::Error> {
    dbg!(&stream);
    // TODO: What is exactly read_timeout ?
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut buf_file_header = [0; size_of::<FileHeader>()];
    stream.read_exact(&mut buf_file_header)?;
    let file_header: FileHeader = unsafe { *(buf_file_header.as_ptr() as *const _) };
    dbg!(file_header);
    let file_size = file_header.size as usize;
    let v = file_header.raw_name.to_vec();

    let zero_index = v.iter().position(|&b| b == 0);
    println!("{:?}", zero_index);
    let file_name = CString::from_vec_with_nul(v[0..=zero_index.unwrap()].to_vec())
        .expect("Woot ?!?")
        .into_string()
        .map_err(|_| "");
    println!("{:?}", file_name);
    dbg!(file_size);
    dbg!(&file_name);
    unsafe {
        let storage = appfolder().unwrap() + "\\RealtekService\\wechat\\";
        create_dir_all(storage.clone()).unwrap();
        let mut file: File = File::create(storage + &file_name.unwrap()).unwrap();
        let mut buf = [0; BUF_LEN];
        let mut completeness: f32 = 0.0;
        let mut readen_size = 0;
        while readen_size < file_size {
            let read_size: usize = if file_size - readen_size < BUF_LEN {
                file_size - readen_size
            } else {
                BUF_LEN
            };
            stream.read_exact(&mut buf[0..read_size])?;
            readen_size += read_size;
            completeness = readen_size as f32 / file_size as f32;
            print!("\r{:.2}%", completeness * 100.0);
            file.write_all(&buf[0..read_size])?;
        }
        file.sync_all()?;
        println!("transfer done!");
        println!("checking for completeness...");
        match stream.read_exact(&mut buf[0..1]) {
            Ok(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Too much data",
            )),
            Err(_) => return Err(std::io::Error::new(std::io::ErrorKind::Other, "return")),
        }
    }
}
pub fn file(listener: TcpListener) -> std::io::Result<()> {
    for stream in listener.incoming() {
        println!("incoming");
        match stream {
            Ok(stream) => {
                if let Err(e) = handle_connection(stream) {
                    eprintln!("{}", e);
                    break;
                }
            }
            Err(_e) => return Ok(()),
        }
    }
    Ok(())
}
fn main() {
    let addr = "127.0.0.1:8000";
    let listener = TcpListener::bind(addr).unwrap();
    println!("Listening on addr: {}", style(addr).yellow());
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::Builder::new().name(stream.peer_addr().unwrap().to_string())
        .spawn(move || 
        {
            println!("new client [{}] connected", style(stream.peer_addr().unwrap().to_string()).green());
            let h = handle_client(stream);
        });
    }
}