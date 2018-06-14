extern crate bitcoin;

use bitcoin::Peer;

fn main() {
    let peer = Peer::connect("dnsseed.bluematt.me:8333").unwrap();
}

