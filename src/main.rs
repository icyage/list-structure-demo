mod dl_list;
use dl_list::*;

fn main() {
    let mut list = DlList::new();
    let one = list.push_back(1);
    let two = list.push_back(2);

    println!("1 = {}", list[one]);
    println!("2 = {}", list[two]);

    println!("1 = {}", list.pop_front().unwrap());
    println!("2 = {}", list.pop_front().unwrap());
    
    println!("false = {}", list.contains(&5));
}
