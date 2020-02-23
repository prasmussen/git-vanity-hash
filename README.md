# git-vanity-hash


## Overview
git-vanity-hash is a command line tool creating commit hashes with a specific prefix


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
