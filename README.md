# CW1-Lockbox

## Instantiate
```rust
pub struct InstantiateMsg {
  pub admin: String,
}
```
## Execute
```rust
pub enum ExecuteMsg {
    CreateLockbox {
        owner: String,
        claims: Vec<Claim>,
        expiration: Scheduled,
        native_token: Option<String>,
        cw20_addr: Option<Addr>
    },

    Reset {},

    Deposit{id: Uint64},

    Receive(Cw20ReceiveMsg),

    Claim{id: Uint64},
}
```
## Query
```rust
pub enum QueryMsg {
    GetLockBox {id: Uint64},
    ListLockBoxes {start_after: Option<u64>, limit: Option<u32>}
}
```
