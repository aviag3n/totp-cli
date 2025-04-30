use std::time::{SystemTime, UNIX_EPOCH};
use base32;


fn htop(key:&str, counter:u128, digits:i32, digest:&str) {

    let key = format!("{}{:=<length$}",key.to_uppercase(),length = ((8 as i32 - "2".chars().count() as i32) % 8) as usize);
    let counter = counter.to_be_bytes();
    let mac = hmac::Hmac::new(key,)

    //let decoded_key = base32::decode(base32::Alphabet::Rfc4648 { padding: true }, key).unwrap();

}

fn main() {
    let since_the_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    println!("{:?}", since_the_epoch);

    //println!("{}{:=<length$}","dgsdg", length = ((8 as i32 - "2".chars().count() as i32) % 8) as usize )
}
