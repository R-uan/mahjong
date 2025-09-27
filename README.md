### Network Components
- Server - handles the incoming client connections.
- Protocol - handles the packets from clients.
- Client - authenticated client state with fetched information about the client.
- ClientManager - handles the upgrade from unauthorized client to client and holds them.

### Game Components
- GameState - stores the game state.
- GameManager - handles the game state and the game logic.

#### Gameplay loop
- Once the match status is ready, it starts the gameplay loop
- The first player (east) draws first and then is prompted to discard

#### Match Manager to Protocol to Client communication
Once the Match status is ongoing
- Client sends a game action
- When the game action is successfully applied (eg: draw, discard):
  - Draw action: the drawn tile is sent to the client that drawn it.
  - Discard action: the discarded tile is sent to all clients.

Once the player connects, automatically sends them their hand.
Once the match status is ready, send a ready MatchStatus to the clients.
Once they receive the status, they may send the same packet if they are ready
that will change their player status to Ready, once all 4 players are ready,
the match status can change to ongoing and start the match.


### TODO
- Player round loop (Draw, Discard, Next Player, Repeat)
- ~~Add Player hand validation (so they can draw the correct amount)~~ 
- ~~Remake protocol package handling~~
- ~~Remove authentication requirement and make it "guest" only~~
- ~~Move client join handling to protocol~~
- ~~Finish the MatchManager > Protocol communication~~
- Figure out how to quickstart the match
- Create Server's game actions (TURN CHANGE, PLAYER ACTION PROMPT, DISCARDED TILE, PLAYER ACTION DONE)


#### Error Codes

#### Server Related Errors [1-50]
- 4 : Failed to bind socket listener.
- 5 : Failed to initialize Log manager.
- 10 : Could not serialize initial player view.

##### Client Related Errors [51-100]
- 54 : Client join request has invalid bytes.
- 55 : Client not found on reconnection request
- 56 : Client attempted an action before sending a connection packet.
- 57 : Client's request's operation is not valid for his current state.

##### Protocol Related Errors [101-150]
- 101 : Packet format is invalid and could not be parsed.
- 102 : Packet kind is not valid.

##### Game Related Errors [151-200]
- 151 : Not all seats are occupied
- 152 : East seat is not occupied
- 153 : West seat is not occupied
- 154 : North seat is not occupied
- 155 : South seat is not occupied
- 161 : Unable to draw tile (Hand Full).
- 162 : Unable to draw tile (Wall Empty).
- 163 : Unable to draw tile (Not player's turn).
- 164 : Unable to discard tile (Tile not in hand)
- 165 : Unable to discard tile (Not player's turn).
