use super::*;

pub struct PeerDb {
    socket: SharedUdpSocket,
    top_node: AtomicArc<Node>,
}

struct Node {
    prefix: XorAddr,
    kind: NodeKind,
}

enum NodeKind {
    Split(u32, AtomicArc<Node>, AtomicArc<Node>),
    Single(Arc<PeerInfo>),
}

impl PeerDb {
    pub fn new(socket: SharedUdpSocket) -> PeerDb {
        PeerDb {
            socket,
            top_node: AtomicArc::new(None),
        }
    }

    pub fn insert(&self, xor_addr: XorAddr, msg: Msg) {
        node_insert(&self.top_node, xor_addr, msg, &self.socket)
    }
}

fn node_insert(
    node: &AtomicArc<Node>,
    xor_addr: XorAddr,
    msg: Msg,
    socket: &SharedUdpSocket,
) {
    loop {
        match node.load(atomic::Ordering::Relaxed) {
            None => {
                let info = PeerInfo::from_msg(&msg);
                let new_node = Node {
                    prefix: xor_addr,
                    kind: NodeKind::Single(info),
                };
                let new_node = Some(Arc::new(new_node));
                let old_node = node.compare_and_swap(None, new_node, atomic::Ordering::Relaxed);
                if let None = old_node {
                    return;
                }
            },
            Some(node_arc) => {
                let xor_diff = node_arc.prefix ^ xor_addr;
                let new_depth = xor_diff.leading_zeros();
                if new_depth < node_arc.prefix_len() {
                    let kind = {
                        let info = PeerInfo::from_msg(&msg);
                        let single = Node {
                            prefix: xor_addr,
                            kind: NodeKind::Single(info),
                        };
                        let moved_node = AtomicArc::from_arc(Some(node_arc.clone()));
                        let single = AtomicArc::new(Some(single));
                        if xor_addr.get_bit(new_depth) {
                            NodeKind::Split(new_depth, moved_node, single)
                        } else {
                            NodeKind::Split(new_depth, single, moved_node)
                        }
                    };
                    let prefix = {
                        let mut prefix = xor_addr;
                        prefix.clear_bits(new_depth);
                        prefix
                    };
                    let new_node = Arc::new(Node { prefix, kind });
                    let old_node = node.compare_and_swap(
                        Some(node_arc.clone()),
                        Some(new_node),
                        atomic::Ordering::Relaxed,
                    );
                    if let Some(old_node_arc) = old_node {
                        if Arc::ptr_eq(&old_node_arc, &node_arc) {
                            return;
                        }
                    }
                } else {
                    match node_arc.kind {
                        NodeKind::Single(ref existing_info) => {
                            existing_info.update(&msg);
                        },
                        NodeKind::Split(_, ref on_zero, ref on_one) => {
                            if xor_addr.get_bit(new_depth) {
                                node_insert(on_one, xor_addr, msg, socket)
                            } else {
                                node_insert(on_zero, xor_addr, msg, socket)
                            }
                        },
                    }
                    return;
                }
            },
        }
    }
}

impl Node {
    fn prefix_len(&self) -> u32 {
        match self.kind {
            NodeKind::Split(prefix_len, _, _) => prefix_len,
            NodeKind::Single(..) => XorAddr::BIT_LEN * 8,
        }
    }
}

