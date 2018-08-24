# Remote storage on `lightstore`

In order to be a replacement for github, youtube et al. it's important for
people to be able to use other peers to host their files. The protocol allows a
node to offer money to another peer to have a file hosted for a certain period
of time, the problem here is one of trust.

If a node wants to upload a file to the network, and have (say) 90% confidence
that the file will be available later, then it must keep upload the file to
enough peers to achieve that confidence level. If the node cannot find any
peers which it rates as significantly trustworthy then it will need to use a
lot of peers instead and the exercise will end up being extremely expensive. As
such, even without a reliable reputation system `lightstore` allows peers to
upload data to the network, but such a system will be necessary to get fees
down to a sane level. The rest of this document describes such a system.

## A reputation system for storage providers

This reputation system is orthogonal to rest of `lightstore` can be added
later. It is also a lot more complex than other parts of the design.

The idea is to base the reputation system on a decentralized prediction market
in the likes of Bitcoin Hivemind or Augur. Prediction markets allow people to
create contracts which pay a fixed amount of money if a certain event occurs.
These contracts are then traded on an open market, and their trading price
relative to their payout can be used as an accurate measure of the probability
of the event occuring. Prediction markets are extremely accurate since any
inaccuracy in a market's price can be exploited for financial gain which, in
the process, corrects the price to its true value.

In `lightstore` we use a prediction market to predict the likelihood of a
hosting provider losing any customer data in a certain time frame. Anyone who
wants to establish themselves as a reliable hosting provider can fund a
contract which pays out if the hosting provider loses customer data. When a
node uploads a file to the hosting provider they receive a receipt, signed by
the provider, which specifies the length of time the provider promises to keep
the file for. If at a later date the node is unable to download the file they
can take this receipt to the prediction market and force the contract to pay
out if the hosting provider can't prove they have the file. Unlike with
Hivemind or Augur this process does not require any human intervention but can
instead be entirely automated. The node who's data was lost can receive some
compensation by buying shares in the contract before exposing the hosting
provider, but the main value provided by this system is that it allows any peer
to check the reliability of any hosting provider simply by checking the current
market price.

