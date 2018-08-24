# `git` workflow

`lightstore` is designed to be used with `git`. 

### Cloning a repo

To clone a repository stored on `lightstore`, we just clone it using its
`lightstore` address.  This address is a public signing key used to sign updates
to the repo.

    $ git clone lsd://9na9s8fn9adsf9ads98f797fa9ndsf89as7ndfas9df8 repo_name

The above command uses the `lightstore` git-remote-helper to find and download
the repo contents. We can also make use of DNS to download a repo with a
well-known name:

    $ git clone lsd://example.com/some_project

This will look in the TXT record for example.com for a line of the form:

    lightstore /some_project 9na9s8fn9adsf9ads98f797fa9ndsf89as7ndfas9df8 

Which gives the public signing key for the repo at /some_project.

### Pushing a repo

In order to upload a local repository to `lightstore`, we first use the
`lightstore` command line tool to create a remote address:

    $ lightstore create --remote origin
    origin set to lsd://9na9s8fn9adsf9ads98f797fa9ndsf89as7ndfas9df8

The --remote flag is optional and sets the given remote to point at the
newly-created address. The address is a public signing key, the secret key to
which is saved to the file
.git/info/lightstore-keys/9na9s8fn9adsf9ads98f797fa9ndsf89as7ndfas9df8

Before pushing to this address a user should review the git config options

    lightstore.max_upload_price
    lightstore.upload_period
    lightstore.upload_price_factor

These tell `lightstore` the length of time to upload for and how to balance the
trade-off between paying less for the upload and having the upload be more
reliable. are set to the user's satisfaction, they can push data to this
address:

    $ git push origin master

This will upload the new refs object under the address signing key, and upload
all commits/objects contained in master.

