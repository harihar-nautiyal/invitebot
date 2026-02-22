# Invite Bot

A matrix bot for mass inviting users to a room.

## Usage

Invite `@invite:unifwe.com` to your room give the bot enough perms to invite
other users and type `!invite #anyroom:matrix.org` Bot will automatically join
that room and start inviting all the users of that room to current room.

## Self-Hosting

You can self-host by building your self

```bash
cargo build --release
```

**Note you need to have environment variables of '.env.example' in order to run the bot**

```bash
cargo run --release
```



