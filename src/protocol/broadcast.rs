use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Broadcast {
    /// Player has drawn a tile. Do not reveal the tile drawn.
    DREW, 
    /// Player discarded a tile. Reveals which tile was discarded.
    DISCARDED, 
    /// Broadcasts the new turn number and who is the next in line.
    TURNCHANGE, 
    /// Broadcasts a winner and that the round has ended.
    WINNER, 
    /// Broadcasts player's tile calls (KAN, CHI, PON). Reveals which sequence/triplet was called.
    CALLS, 
    /// Fatal server error broadcast to end the match.
    ERROR, 
}