## `lightstore`: yet another file-sharing network

This is the homepage and documentation for `lightstore`, a work-in-progress
file-sharing network.

Lightstore differs from networks like IPFS and Bittorrent by allowing peers to
charge for the services they provide. All activity on the network moves debt
between peers, and this debt can be cleared via bitcoin lightning network
microtransactions.

Unlike other cryptocurrency-based file-sharing networks, `lightstore` uses an
existing, well-established currency (Bitcoin), and prices are set by market
forces. The market is designed so that peers will compete to fulfil each others
requests as quickly and cheaply as possible.

`lightstore` is designed to be used with git, ie. peers share git repositories
and commits rather than just individual files. The `git` command-line tool,
used with the lightstore plugin, provides the interface for downloading files
and also allows you to push updates to the network.

`lightstore` is still in the early concept stage of development. If you're
interested in learning more or in helping out, please read the docs below. You
can also [support me on Patreon](https://patreon.com/canndrew).

### Documentation

  * [Overview](doc/overview.md)
  * [`git` workflow](doc/git-workflow.md)
  * [Remote storage reputation system](doc/remote-storage.md)

### Source code

The source code is currently [available on
github](https://github.com/canndrew/lightstore).

