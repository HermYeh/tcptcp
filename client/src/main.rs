mod clients;
use clients::{Protocol, Response};





fn main() {
    match Protocol::connect("69.117.108.66:8000").and_then(|mut client| {
        client.filetransfer()?;
        Ok(client)
    }).and_then(|mut client| client.read_message::<Response>()).map(|resp| println!("{}", resp.message())){
        Ok(_)=>{}
        Err(_)=>{
           
        }
    }
    
    
}