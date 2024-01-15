# What is this?
A web-app where you prove statements using logic and chat-gpt.\
The frontend uses react-force-graph to visualize the statements and their relations.

## Client (argue-react)
### Setup:
1. Install nodejs
2. `npm install`
3. configure .env / websocket address in Game.tsx:86

### Develop: `npm start`

### Production: `npm run build`
Note that this frontend depends on a corresponding backend to handle the websocket requests and the game logic, including consulting chat-gpt.

## Server (argue-rust)
## Setup
1. Dependency: rust
2. Put your openai api key in `openai.key`
3. Configuration: SOCKET_ADRESS in `src/main.rs`
4. `cargo run`

## Documentation

## Client ws-messages
```json
"GetGameState"
{"Add":{"statement":"..."}}
{"Delete":{"id":[0,0]}}
{"Edit":{"id":[0,0],"statement":"..."}}
{"Link":{"premise":[0,0],"conclusion":[0,0]}}
{"Unlink":{"premise":[0,0],"conclusion":[0,0]}}
{"ProveDirect":{"id":[0,0]}}
{"ProveImplication":{"id":[0,0]}}
```

Example:
```json
{"Add":{"statement":"Socrates is a man."}}
{"Add": {"statement":"Every man is mortal."}}
{"Add": {"statement": "Socrates is mortal."}}
{"Link":{"premise":[1,0],"conclusion":[3,0]}}
{"Link":{"premise":[2,0],"conclusion":[3,0]}}
{"ProveImplication":{"id":[3,0]}}
{"ProveDirect":{"id":[1,0]}}
{"ProveDirect":{"id":[2,0]}}
```

## Server ws-messages
STATE = None|DirectlyProven|ImpliedUnproven|ImpliedProven

```json
{"NewNodeId":{"id":[0,0]}}
{"GameState":{"statements":[{"id":[0,0],"statement": "...","state": "STATE","parents":[[0,0]],"children": [[0,0]],},]}, "root": [0,0]}
{"Comment":{"id":[0,0],"comment": "...","success": false}}
"Win"
{"AICooldown":{"seconds":15}}
{"Error":{"NoSuchNode":[0,0]}}
{"Error":"RemoveRoot"}
{"Error":{"AddExistingLink":{"child":[0,0],"parent":[0,0]}}}
{"Error":{"RemoveNonExistentLink":{"child":[0,0],"parent":[0,0]}}}
```
