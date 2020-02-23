# git-vanity-hash


## Overview
git-vanity-hash is a command line tool for creating commit hashes with a specific prefix


## FAQ
```
Q: Is this a good idea?
A: No

Q: Will this break git tooling?
A: Probably (see [how it works](#how-it-works))
```


## Usage
```
git-vanity-hash <mode> <prefix>

mode
    find        Find and print hash (read-only)
    update      Find and update HEAD with found hash

prefix
    A hexadecimal string the hash should start with
```


## Examples
###### Find hash
```
$ git-vanity-hash find cafe
Found hash: cafeacc13453e3d5a3fc8d0c57bccc702e92917f
```

###### Update HEAD
```
$ git-vanity-hash update beef
Found hash: beefe706a9612d639e6a176703ad61bfd5f6df10
HEAD updated
```

###### Full example
```
$ git init
Initialized empty Git repository in /Users/user/cool-project/.git/

$ git add README.md

$ git commit -m "Add readme"
[master (root-commit) 6af06ae] Add readme
 1 file changed, 1 insertion(+)
 create mode 100644 README.md

$ PAGER=cat git log
commit 6af06aeb70482ba69e5c85225f8c4a0e98cbd942 (HEAD -> master)
Author: Petter Rasmussen <petter@hask.no>
Date:   Sun Feb 23 14:11:19 2020 +0100

    Add readme

$ git-vanity-hash update cafe
Found hash: cafe7f3302e66fef6428029563534ff2d8d0bc4f
HEAD updated

$ PAGER=cat git log
commit cafe7f3302e66fef6428029563534ff2d8d0bc4f (HEAD -> master)
Author: Petter Rasmussen <petter@hask.no>
Date:   Sun Feb 23 14:11:19 2020 +0100

    Add readme
```


## How it works
The commit object is read using `git cat-file commit HEAD`.
An example how it looks:
```
tree bd2171194b1f31c586da1ee133a845810e303a23
parent 000000757d84e66644069f387f30f657faee4187
author Petter Rasmussen <petter@hask.no> 1582460250 +0100
committer Petter Rasmussen <petter@hask.no> 1582460640 +0100

Add README
```

A new `vanity` header and value is added.
An example how it looks after the `vanity` headers is added:
```
tree bd2171194b1f31c586da1ee133a845810e303a23
parent 000000757d84e66644069f387f30f657faee4187
author Petter Rasmussen <petter@hask.no> 1582460250 +0100
committer Petter Rasmussen <petter@hask.no> 1582460640 +0100
vanity 7-16488

Add README
```

The string is hashed (see [commit_info.rs](src/git_vanity_hash/commit_info.rs) for more details) and checked if it has the wanted prefix.
The `vanity` value is increased until a matching hash prefix is found.

When a match is found the new commit object is written to the object database with `git hash-object -t commit --stdin`
And then HEAD is changed to point to the new object with `git update-ref HEAD <hash>`


The `vanity` header will normally not show when using basic git commands, but can be seen using i.e. `git cat-file commit HEAD`.
Note that the default git tools has support for extra headers, but there is no guarantee that this won't break 3rd tools.


## Similar projects
* https://github.com/mattbaker/git-vanity-sha
* https://github.com/tochev/git-vanity
