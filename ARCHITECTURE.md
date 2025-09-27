

# TOGETHER APP

This is a worldcoin specific mini-app that allows two users to prove that they were together.

## Engineering Idea

We'll initially explore this by using a sound-encoded message where one device is playing the sound, and another "proximally close" device is listening for the sound.

Since World is such a widely distributed network with many devices and users in different countries, sound transmission is widely available as a means of communication, whereas NFC/location/bluetooth/apple-specific-features may not be.

This audio encoded message is going to encode the wallet address of the user (and maybe a rolling code, timestamp, etc.)
When the other user's device hears the message, their device will decode the message and sign a signature of the two addresses and timestamp. This sends the signature to our backend, and our backend will verify the signature and submit the transaction through a sponsored transaction.

## Shared things

url for the app is `https://togetherapp.app`
frontend will be `https://miniapp.togetherapp.app`
- hosted on vercel
backend will be `https://api.togetherapp.app`
- hosted on railway

## Frontend
This uses the worldcoin mini-app template. It's mobile only and within this context has information about the logged in user.
- username
- wallet address

Ideally we'd use Nextjs + shadcn + tailwind, but we'll prioritize using packages within the template

We'll fetch the user's together data from our database
- number of connections
- connections to who and when
  - include usernames of connections? these can be optimistic (stale is ~ok) and come from username_cache table

For now, we'll have the app be pretty debug-heavy where we show their address, and then we'll have a randomly generated and shown "together-with" address, and then a "Pair" button that calls the server to attest these two were together.
- in the future this will be a different mechanism but for now this will be good to test the backend/db/etc.



## Backend
Rust based. Postgres.

- User-profile data viewer
  - when a user visits their page (or another user's page), we'll fetch the connection data from our database for that user's connections.
- Transaction sender
  - verifies a received attestation and if valid, sends a transaction
- Attestation watcher
  - watches for attestations emitted, then populates the database

## Contracts

We'll make a simple contract around emitting "Together" attestations.
(address,address,timestamp)

- the "together" function needs an AuthData for a permissioned signer.
- we'll have permissioned signers for the "together" function.
- we'll have viewer functions for helpers around:
  - check(address, address) -> (bool, timestamp)
  - get_together_count(address) -> u64
  - together_list_at_index(address, index) -> (address, timestamp)

## Database
address,address,timestamp

We want to quickly be able to get the number of connections for a user, and to get the connections for a user so that we can display that on their profile page.






