extern crate tcpgen;
use tcpgen::{TCPList, TCPType};
use std::io::BufRead;

fn main() {
    let stdin = std::io::stdin();
    let mut lines = stdin.lock().lines();
    let tcp_list = TCPList::new("./");
    let mut count = 0;
    for (tcp_type, list) in tcp_list.types.iter() {
        if *tcp_type == TCPType::Unknown {
            println!("{} types with an unknown category", list.len());
        }
        count += list.len();
    }

    println!("Welcome to the TCP random generator, there are {} types", count);
    println!("Please input how many you want to generate, 0 to exit");
    while let Some(Ok(line)) = lines.next() {
        let num = line.parse().unwrap_or(0);
        if num == 0 {
            break;
        }
        for _ in 0..num {
            let tcp = tcp_list.gen();
            println!("{}", tcp);
        }
        println!("Would you like more TCPs? Input how many if so, 0 to exit");
    }
}
