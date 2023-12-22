use rand::RngCore;
use sqids::Sqids;
use std::sync::OnceLock;

static SQID: OnceLock<Sqids> = OnceLock::new();

fn get_sqids() -> &'static Sqids {
    SQID.get_or_init(|| Sqids::new(None).expect("Could not generate Sqid object"))
}

pub fn unique_id() -> String {
    let mut rng = rand::thread_rng();
    get_sqids()
        .encode(&[rng.next_u64()])
        .expect("Could not generate sqid")
}
