#![feature(decl_macro, proc_macro_hygiene)]

#[macro_use]
extern crate log;
extern crate env_logger;

mod api_provider;
mod bahnapi;
mod spaceapi;

fn main() -> Result<(), failure::Error> {
    //env_logger::init();
    //bahnapi::search("CBase".to_string());
    // 2000km is the default magic number for max travel from station to space
    //bahnapi::nearby(54.303623,10.124862,Some(2000), Some(10));
    //println!("{:?}", spaceapi::get_spaces().unwrap());
    api_provider::start_api();
    Ok(())
}
