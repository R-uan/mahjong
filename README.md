### Network Components
- Server - handles the incoming client connections.
- Protocol - handles the packets from clients.
- Client - authenticated client state with fetched information about the client.
- ClientManager - handles the upgrade from unauthorized client to client and holds them.

### Game Components
- GameState - stores the game state.
- GameManager - handles the game state and the game logic.

#### Gameplay loop
- Game starts, east if the first player.
- East draw a tile, server waits for his discard
- Once discarded, the server checks for win condition, if none, go to next player
- next player draw...




### TODO
- Player round loop (Draw, Discard, Next Player, Repeat)
- Add Player hand validation (so they can draw the correct amount) 

#### Error Codes

##### Game Related Errors [151-200]
- 151 : Unable to draw tile (Hand Full).
- 152 : Unable to draw tile (Wall Empty).

