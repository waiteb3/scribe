Copyright (C) 2020 Brandon Waite - All rights reserved

## Scribe

`scribe` is a tool that securely records you shell history and syncs it across multiple machines.

Acquiring a license: Currently granted on request

Terms and conditions:
- By using or reading the software, you agree to license and conditions of use, and acknowledge that you have read and understood any agreements to using the software.
- The software comes with no provided waranty or guarantees, express or implied, and is provided "AS-IS". All rights are reserved.
- The software _is not_ open source, meaning it is not permitted to be remix or redistributed outside of official channels.
- The software _is_ source available, so you _may_ build the software yourself if desired.
- Data created by this software is permitted for personal use only.
- License and access to the software or associated services is not guaranteed to be permanent.

Personal notes:
I am not asking you to blindly trust a binary, so inspect and audit the code until your heart is content. That's why it's source available.

### Install

```bash
make init
make install
```

After running the install command, add this to your rc file (zshrc, bashrc, etc)
```
source <( scribe init )
```
