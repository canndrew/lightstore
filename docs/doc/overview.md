# `lightstore` - Overview

`lightstore` is a P2P file-sharing network where peers are compensated in
bitcoin for the services they provide to the network. Peers are able to charge
each other for bandwidth, remote storage, or any other services they provide.
Prices are set by market forces and users can set their own pricing structures
in order to tune the performance of the network to their specific needs.

The design goals of the system are to be:

  * Fast
    Small downloads should be able to complete in one round-trip to the
    network. If the network can't compete with HTTP then nobody will end up
    using it.

  * Reliable
    Rare and obscure files should be able to survive long-term on the network.

  * Sustainable
    Peers should be compensated for any work they do in providing for the
    network. It should not be possible for peers to systematically freeload off
    of one another.

## Economics of `lightstore`

### Transations and debt

Every connection between two peers on the network has an associated balance
known to those two nodes. This is an amount of bitcoin representing the debt
owned by one peer to another. Nodes charge each other fees to receive and
process messages, whenever any message is exchanged on the network both peers
update their recorded balance accordingly. This balance is not
cryptographically secure, it's simply a counter on both computers. Any peer can
disconnect, go offline, or start ignoring any other peer at any time. If a peer
reconnects to another peer using the same public key, both peers should make an
effort to recall the previous balance.

Balances can be updated via bitcoin lightning network transactions. Even though
these transactions are cheap, they're not so cheap that they can be sent with
every message without causing massive overhead, hence why we use a seperate
non-cryptographically-secure balance counter. Nodes are free to decide how much
debt they're willing to tolerate from a peer before they start ignoring them.
This will typically be very small amounts, depedent on the trade-off a
node makes between trying to ensure they get paid, encouraging business, and
tolerating the overhead of transactions. Nodes can also use bitcoin mining
attempts as currency in order to make it possible for nodes with empty wallets
to use the network.

### Pricing

In order to prevent leaching, all network activity costs money. When nodes
initially establish a connection the first thing they tell each other is their
fee structure. Either party can update their fees at any time.

Nodes charge money to be sent data. This prevents peers from doing anything
that might waste a node's bandwidth unnecessarily.

Every request on the network comes with an offer of money, the value of which
can be dependent on characteristics of the reply. For example, a DHT lookup
request includes an amount of bitcoin and a decay rate which specifies how
quickly the offer shrinks the longer it takes a peer to reply. The offer can
also depend on whether the peer replies with the desired data, whether they
reply with contact information for peers closer to the key in xor-space,
exactly how close to the key these peers are, etc.

In short, money is used everywhere at every level of the protocol.  For more
detailed information on how this works, see some of the other docs which delve
into specific parts of the network in more detail.

