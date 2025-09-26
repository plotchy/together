

Our database will be designed around this usecase:
- Together is a attestation system where two users can attest to being together at a given timestamp.
- We can watch for attestations to be emitted, and insert these into our database
- when a user visits the website, we read which attestations they've been a part of
  - total count
  - users they've been together with and at what timestamp
