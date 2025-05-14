pub fn make_addr(id: u64) -> mc::Address {
    mc::Address::new(format!("n{id}"), "rsm")
}
