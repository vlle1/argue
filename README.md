# What is this?
A web-app where you prove statements using logic and chat-gpt.\
The frontend uses react-force-graph to visualize the statements and their relations.

### Setup:
1. Dependencies: npm, rust
2. in `.\argue-react`:
   1. `npm install`
   2. configure .env
   3. `npm run build`
3. configure .env
4. Put your openai api key in new file `openai.key`

### run Client only
run `npm start` in argue-react

### run Server
run `cargo run` in root folder


## Documentation
The websocket url used determines if the client is playing privately or is using a public pad.\
The ws url has an optional statement and an optional private parameter.
It looks like this: `ws://BASE_URL/ws[/:statement][?private=:bool]`\
Example: `ws://BASE_URL/ws/This+is+the+statement?private=true`
The default `ws://BASE_URL/ws` is a public pad with root empty initial root statement.

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
