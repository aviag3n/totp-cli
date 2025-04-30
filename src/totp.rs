use std::time::{SystemTime, UNIX_EPOCH};
use base32;
use hmac::{Hmac, Mac};
use sha1::Sha1;
use keyring::{Entry, Result};


type HmacSha1 = Hmac<Sha1>;



fn compute_hmac(key: &[u8],counter: u64 ) -> Vec<u8> {
    let counter_bytes = counter.to_be_bytes();

    let mut mac = HmacSha1::new_from_slice(key).expect("HMAC can take any key size");
    mac.update(&counter_bytes);
    mac.finalize().into_bytes().to_vec()
}
fn htop(key:&str, counter:u64, digits:usize) -> String {

    let key = format!("{k:=<length$}", k=key.to_uppercase(), length = ((8 as i32 - "2".chars().count() as i32) % 8) as usize);

   //println!("key : {}",key);

    let decoded_key = base32::decode(base32::Alphabet::Rfc4648 { padding: true }, &key).unwrap_or_else(|| {return "INVALID_CODE_INPUTTED".into();});

    let error_var: Vec<u8> = "INVALID_CODE_INPUTTED".into();

    if decoded_key == error_var {
        return "INVALID".to_string();
    }

    //let hex: String = decoded_key.iter().map(|b| format!("{:02x}", b)).collect();
    //println!("dec : {}", hex);

    //let counter_bytes = counter.to_be_bytes();

    //let hex: String = counter_bytes.iter().map(|b| format!("{:02x}", b)).collect();
    //let padded = format!("{:0>16}", hex);
    //println!("cou : {}",padded);


    let mac = compute_hmac(&decoded_key, counter);

    //let hex: String = mac.iter().map(|b| format!("{:02x}", b)).collect();
    //println!("mac : {}", hex);

    let offset = (mac[mac.len() - 1] & 0x0f) as usize;
    let slice = &mac[offset..offset + 4]; // mac is a Vec<u8>
    let binary = (u32::from_be_bytes(slice.try_into().unwrap()) & 0x7fffffff).to_string();



    let result = binary[binary.chars().count() - digits..].to_string();

    return format!("{r:0<d$}", r=result, d=digits);



}

pub fn totp(key:&str, time_step:u64, digits:usize) -> String {

    let time = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("Time went backwards").as_secs() as u64;

    return htop(key, time / time_step, digits);
}

#[allow(dead_code)]
fn get_entries(service:&str, user:&str) -> Result<()> {
    let entry = Entry::new(service, user)?;
    entry.set_password("topS3cr3tP4$$w0rd")?;

    let password = entry.get_password()?;
    println!("My password is '{}'", password);
    entry.delete_credential()?;

    Ok(())
}

#[allow(dead_code)]
fn create_entry(service:&str, user:&str) -> Result<()> {

    let entry = Entry::new(service, user)?;
    entry.set_password("topS3cr3tP4$$w0rd")?;

    Ok(())
}


