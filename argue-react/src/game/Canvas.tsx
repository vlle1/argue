import { ForceGraph2D } from "react-force-graph";
import { css_val } from "../util/util";
import { StatementState } from "./Game";
import { useEffect, useState } from "react";

export type Index = string;
export type Node = {
  id: Index;
  statement: Index;
  state: StatementState;
  fx?: number;
  fy?: number;
  fz?: number;
};
export type Link = {
  source: Index;
  target: Index;
};
export type GraphData = {
  //graph data has many nodes
  nodes: Node[];
  links: Link[];
  rootId: Index;
};
//make sure to keep meta-data as position etc.
function updateGraph(
  { nodes: new_nodes, links, rootId }: GraphData,
  setData: React.Dispatch<React.SetStateAction<GraphData>>
) {
  console.log("updateGraph", new_nodes, links, rootId);
  setData(({ nodes: used_nodes }: GraphData) => {
    for (let new_node of new_nodes) {
      let used_node = used_nodes.find((n) => n.id === new_node.id);
      if (used_node) {
        //update node
        used_node.statement = new_node.statement;
        used_node.state = new_node.state;
      } else {
        //add node
        new_node.fx = 0;
        new_node.fy = 0;
        new_node.fz = 0;
        used_nodes.push(new_node);
      }
    }
    used_nodes = used_nodes.filter((n) => new_nodes.find((m) => m.id === n.id));
    /* Farben reinschummeln !?
    for (let link of links) {
      let source = new_nodes.find((n) => n.id === link.source);
      let target = new_nodes.find((n) => n.id === link.target);
      if (!source || !target) continue;
      link.color = edgeColor(source, target);
    }
    */
    return { nodes: used_nodes, links, rootId };
  });
}
function nodeColor(node: Node): string {
  switch (node.state) {
    case "DirectlyProven": {
      return css_val("--fact-color");
    }
    case "ImpliedProven": {
      return css_val("--implied-proven-color");
    }
    case "ImpliedUnproven": {
      return css_val("--implied-unproven-color");
    }
    case "None": {
      return css_val("--none-color");
      }
  }
}
function edgeState(
  source: Node,
  target: Node
): "implied-proven" | "implied-unproven" | "none" {
  if (target.state === "ImpliedProven") {
    return "implied-proven";
  }
  if (target.state === "ImpliedUnproven") {
    if (source.state === "ImpliedProven" || source.state === "DirectlyProven") {
      return "implied-proven";
    } else return "implied-unproven";
  }
  return "none";
}

const Canvas = ({
  graphData,
  onBackgroundRightClick,
  onNodeRightClick,
  linkNodes,
  onLinkRightClick,
  directProve,
}: {
  graphData: GraphData;
  //callbacks:
  onBackgroundRightClick: (e: MouseEvent) => void;
  onNodeRightClick: (e: MouseEvent, id: Index) => void;
  linkNodes: (from: Index, to: Index) => void;
  onLinkRightClick: (from: Index, to: Index) => void;
  directProve: (id: Index) => void;
}) => {
  const [data, setData] = useState<GraphData>(graphData);
  useEffect(() => updateGraph(graphData, setData), [graphData]);
  let [selected, select] = useState<{ id: Index; x: number; y: number } | null>(
    null
  );
  return (
    <ForceGraph2D
      graphData={data}
      nodeCanvasObject={(node, ctx) => {
        const label = node.statement;
        const fontSize = node.id == data.rootId ? 20 : 10;
        ctx.font = `${fontSize}px Segoe UI`;
        const textWidth = ctx.measureText(label).width;
        const bckgDimensions = [textWidth, fontSize].map(
          (n) => n + fontSize * 0.2
        ); // some padding

        ctx.fillStyle =
          node.id == selected?.id
            ? css_val("--selected-node-bg")
            : css_val("--node-bg");
        let padding = 10;
        ctx.fillRect(
          (node.x || 0) - bckgDimensions[0] / 2 - padding,
          (node.y || 0) - bckgDimensions[1] / 2 - padding,
          bckgDimensions[0] + 2 * padding,
          bckgDimensions[1] + 2 * padding
        );
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillStyle = nodeColor(node);
        ctx.fillText(label, node.x || 0, node.y || 0);
        node.__bckgDimensions = bckgDimensions; // to re-use in nodePointerAreaPaint
      }}
      nodePointerAreaPaint={(node, color, ctx) => {
        //hitbox (?)
        ctx.fillStyle = color;
        const bckgDimensions = node.__bckgDimensions;
        let padding = 10;
        bckgDimensions &&
          ctx.fillRect(
            (node.x || 0) - bckgDimensions[0] / 2 - padding,
            (node.y || 0) - bckgDimensions[1] / 2 - padding,
            bckgDimensions[0] + 2 * padding,
            bckgDimensions[1] + 2 * padding
          );
      }}
      onNodeRightClick={(node, e) => onNodeRightClick(e, node.id)}
      onNodeClick={(node, e) => {
        if (e.shiftKey) {
          //unfix nodes
          delete node.fx;
          delete node.fy;
          delete node.fz;
        } else {
          if (selected) {
            if (selected.id != node.id) {
              linkNodes(selected.id, node.id);
            } else {
              directProve(node.id);
            }
            select(null);
          } else {
            select({ id: node.id, x: node.x || 0, y: node.y || 0 });
          }
        }
      }}
      onNodeHover={(node, prevNode) => {
        //note: node, prevNode can be null
        //TODO...
      }}
      onLinkRightClick={(link) => {
        let source: Node = link.source as Node;
        let target: Node = link.target as Node;
        onLinkRightClick(source.id, target.id);
      }}
      onNodeDragEnd={(node) => {
        //auto fix the position:
        node.fx = node.x;
        node.fy = node.y;
        node.fz = node.z;
      }}
      onBackgroundRightClick={onBackgroundRightClick}
      onBackgroundClick={(e) => {
        if (selected) {
          select(null);
        } else {
          //TODO create new node?
        }
      }}
      linkDirectionalParticles={10}
      linkDirectionalParticleColor={(link) => {
        let source = link.source as Node;
        let target = link.target as Node;
        return css_val(`--${edgeState(source, target)}-color`);
      }}
      linkDirectionalParticleSpeed={(link) => {
        //proven > implied > none
        let source = link.source as Node;
        let target = link.target as Node;
        let state = edgeState(source, target);
        if (state === "implied-proven") {
          return 0.004;
        }
        if (state === "implied-unproven") {
          return 0.002;
        }
        return 0.001;
      }}
      linkDirectionalParticleWidth={(link) => {
        let source = link.source as Node;
        let target = link.target as Node;
        return edgeState(source, target) == "none" ? 3 : 5;
      }}
      linkCurvature={0.1}
    />
  );
};

export { Canvas };
