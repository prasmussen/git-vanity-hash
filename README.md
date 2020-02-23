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
