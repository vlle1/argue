import { Canvas, GraphData, Index, Node, Link } from "./Canvas";
import React, { useState, useCallback, useEffect } from "react";
import useWebSocket, { ReadyState } from "react-use-websocket";

export type StatementState =
  | "None"
  | "DirectlyProven"
  | "ImpliedUnproven"
  | "ImpliedProven";
type StatementDTO = {
  id: IndexDTO;
  statement: string;
  state: StatementState;
  parents: IndexDTO[];
  children: IndexDTO[];
};
type ServerGameState = {
  statements: StatementDTO[];
  root: IndexDTO;
};
type IndexDTO = [number, number];
type LinkDTO = { child: IndexDTO; parent: IndexDTO };
type ServerError =
  | { NoSuchNode: IndexDTO }
  | "RemoveRoot"
  | { AddExistingLink: { child: IndexDTO; parent: IndexDTO } }
  | { RemoveExistentLink: LinkDTO };

type ServerMessage =
  | NewNodeIdDTO
  | GameStateDTO
  | CommentDTO
  | AICooldownDTO
  | ErrorDTO
  | "Win";

type NewNodeIdDTO = { NewNodeId: { id: IndexDTO } };
type GameStateDTO = {
  GameState: { statements: StatementDTO[]; root: IndexDTO };
};
type CommentDTO = {
  Comment: { id: IndexDTO; comment: string; success: boolean };
};
type AICooldownDTO = { AICooldown: { seconds: number } };
type ErrorDTO = { Error: ServerError };

type ClientMessage =
  | { Add: { statement: string } }
  | { Delete: { id: IndexDTO } }
  | { Edit: { id: IndexDTO; statement: string } }
  | { Link: { premise: IndexDTO; conclusion: IndexDTO } }
  | { Unlink: { premise: IndexDTO; conclusion: IndexDTO } }
  | { ProveDirect: { id: IndexDTO } }
  | { ProveImplication: { id: IndexDTO } };

function toIndex(index: IndexDTO): Index {
  return `${index[0]},${index[1]}`;
}
function toIndexDTO(index: Index): IndexDTO {
  let [i, gen] = index.split(",");
  return [parseInt(i), parseInt(gen)] as IndexDTO;
}
function toGraphData(state: ServerGameState): GraphData {
  let clientStateNodes: Node[] = [];
  let clientStateLinks: Link[] = [];
  state.statements.forEach((s) => {
    clientStateNodes.push({
      id: toIndex(s.id),
      statement: s.statement,
      state: s.state,
    });
    s.children.forEach((c) => {
      clientStateLinks.push({ source: toIndex(c), target: toIndex(s.id) });
    });
  });

  return {
    nodes: clientStateNodes,
    links: clientStateLinks,
    rootId: toIndex(state.root),
  };
}

const Game = ({ root_statement }: { root_statement: string }) => {
  // get address from env
  const address = process.env.REACT_APP_WEBSOCKET_URL as string;
  const [socketUrl, setSocketUrl] = useState(address);
  const { sendJsonMessage, readyState, lastJsonMessage } = useWebSocket(
    socketUrl,
    {
      onOpen: () => {
        console.log("WebSocket connection opened.");
        let setCorrectRoot: ClientMessage = {
          Edit: { id: [0, 0], statement: root_statement },
        };
        sendJsonMessage(setCorrectRoot);
      },
      shouldReconnect: (_) => false,
    }
  );
  const connectionStatus = {
    [ReadyState.CONNECTING]: "Connecting",
    [ReadyState.OPEN]: "Open",
    [ReadyState.CLOSING]: "Closing",
    [ReadyState.CLOSED]: "Closed",
    [ReadyState.UNINSTANTIATED]: "Uninstantiated",
  }[readyState];

  const [graphData, setGraphData] = useState<GraphData>({
    nodes: [],
    links: [],
    rootId: "",
  });
  const send_message = (expression: ClientMessage) => {
    if (readyState == ReadyState.OPEN) {
      console.log("sending: " + expression);
      sendJsonMessage(expression);
    } else {
      console.log(`Socket not connected while trying to send ${expression}`);
    }
  };
  const add_statement = (statement: string) => {
    send_message({ Add: { statement } });
  };
  useEffect(() => {
    //on message.
    if (!lastJsonMessage) return;
    const message: ServerMessage = lastJsonMessage as ServerMessage;
    console.log("Received message:", message);
    if (message == "Win") {
      console.log("You won!");
      //TODO show win screen.
    } else {
      switch (true) {
        case "NewNodeId" in message: {
          let id = (message as NewNodeIdDTO).NewNodeId.id;
          //TODO fix position of new node.
          break;
        }
        case "GameState" in message: {
          let state = (message as GameStateDTO).GameState;
          setGraphData(toGraphData(state));
          break;
        }
        case "Comment" in message: {
          let { id, comment, success } = (message as CommentDTO).Comment;
          console.log(
            `Node ${id}: ${comment} \n => Action ${
              success ? "successful" : "unsuccessful"
            }.`
          );
          break;
        }
        case "AICooldown" in message: {
          let { seconds } = (message as AICooldownDTO).AICooldown;
          console.log(`AI cooldown: ${seconds} seconds.`);
          //TODO visualize ai cooldown.
          break;
        }
        case "Error" in message: {
          let error = (message as ErrorDTO).Error;
          console.log(`Server Error: ${error}`);
          //TODO display error in some better way.
          break;
        }
        default: {
          console.log("Unknown message:", message);
          break;
        }
      }
    }
  }, [lastJsonMessage]);
  return (
    <div>
      <h1 className="abs-title">Argue: “{root_statement}“ </h1>
      <Canvas
        graphData={graphData}
        onBackgroundRightClick={() => {
          //TODO show VoidMenu
        }}
        onNodeRightClick={(e, node) => {
          send_message({ ProveImplication: { id: toIndexDTO(node) } });
          //TODO show NodeMenu
        }}
        linkNodes={(fromNode, toNode) => {
          send_message({
            Link: {
              premise: toIndexDTO(fromNode),
              conclusion: toIndexDTO(toNode),
            },
          });
        }}
        onLinkRightClick={(fromNode, toNode) => {
          send_message({
            Unlink: {
              premise: toIndexDTO(fromNode),
              conclusion: toIndexDTO(toNode),
            },
          });
        }}
        directProve={(node) => {
          send_message({ ProveDirect: { id: toIndexDTO(node) } });
        }}
      ></Canvas>
      <div className="abs-debug">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            const formData = new FormData(e.target as HTMLFormElement);
            add_statement(formData.get("expression")?.toString() || "");
          }}
        >
          <input
            type="text"
            placeholder="New Statement (to create a new node)"
            name="expression"
            className="abs-in"
          ></input>
        </form>
      </div>
    </div>
  );
};

export default Game;
