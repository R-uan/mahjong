### Network Components
- Server - handles the incoming client connections.
- Protocol - handles the packets from clients.
- Client - authenticated client state with fetched information about the client.
- ClientManager - handles the upgrade from unauthorized client to client and holds them.

### Game Components
- GameState - stores the game state.
- GameManager - handles the game state and the game logic.


Server -> ClientManager -> Client

Client -> Protocol

Protocol -> GameManager -> GameState

GameManager -> Server -> Protocol

### Authentication Pipeline
- New client connects
- ClientManager waits for authentication
  - Client has 5 attempts
  - If authenticated, Client is created
  
