Each user when they first log into the app gets an incrementing ID. (starts at 1).

this is tied to their wallet address through this verified miniapp.

A user is able to type in the ID of another user to send out a "pending connection" for them.
- the other user doesn't see this, they need to type in the other user's ID to complete the connection.

When our DB sees that two users have completed a connection, we'll send a transaction to the Together contract to attest that they connected.

These pending connections will expire after 10 minutes.
- our backend checks the db for these every ~5 seconds.

SO we need to have incrementing unique IDs for each user.
we need a pending connection table (id, from_user_id, to_user_id, created_at, expires_at)

We need a backend bin that checks these pending connections tables and checks matches, removes expiries, etc.
- this should be a pretty lean table since all the data will be cleaned up. checked matches dont need to be stored (they're in the attestations table), and expired pendings dont need to be stored (theyre inactionable).
- when the bin finds a match, itll query the user tables to get the wallet addresses and then send the tx to the contract.

the current attest route for the server is not used in this new system. instead we'll have this pending connection system
- we should probably have a optimistic_connections table so that if two users both have a pending connection (and our db sent the tx, but the tx is taking a few minutes to land) we can optimistically show the users that they're connected!

