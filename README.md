# CW1-Lockbox
CW1 Lockbox is designed to keep a certain amount of native coins or CW20 tokens locked in the contract for a predetermined amount of time before those with the claim rights can withdraw their designated share of coins/tokens.

## Instantiate
```rust
pub struct InstantiateMsg {
}
```
## Execute

```rust
pub enum ExecuteMsg {
  CreateLockbox {
    owner: String,
    raw_claims: Vec<RawClaim>,
    expiration: Scheduled,
    native_token: Option<String>,
    cw20_addr: Option<String>
  },

  Reset { id: Uint64 },

  Deposit { id: Uint64 },

  Receive(Cw20ReceiveMsg),

  Claim { id: Uint64 },
}
```
## Query
```rust
pub enum QueryMsg {
  GetLockBox {id: Uint64},

  ListLockBoxes {start_after: Option<u64>, limit: Option<u32>}
}
```
