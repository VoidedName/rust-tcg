import React, { ReactNode } from 'react';
import {Edge, Graph, Node} from "./Graph.ts";

export type Location = {
  x: number,
  y: number,
};

export type GraphLayout = {
  [node: number]: Location
};

export type NodeRenderer<TData,
  TGraph extends Graph<TData>,
  > =
  (
    graph: TGraph,
    layout: GraphLayout,
    node: Node<TData>,
  ) => ReactNode;

export type EdgeRenderer<TData,
  TGraph extends Graph<TData>,
  > =
  (
    graph: TGraph,
    layout: GraphLayout,
    edge: Edge,
  ) => ReactNode;

export type GraphLayouter<TData,
  TGraph extends Graph<TData>,
  > =
  (
    graph: TGraph,
    width: number,
    height: number,
  ) => GraphLayout;

export type GraphRendererProps<TData, TGraph extends Graph<TData>> = {
  graph: TGraph
  renderNode: NodeRenderer<TData, TGraph>
  renderEdge: EdgeRenderer<TData, TGraph>
  layout: GraphLayout
  defs: ReactNode,
};

export default function BaseGraphRenderer<TData, TGraph extends Graph<TData>>({
  graph,
  renderNode,
  renderEdge,
  layout,
  defs,
}: GraphRendererProps<TData, TGraph>) {
  return (
    <svg>
      <defs>{defs}</defs>
      {graph.edges.map((edge) => <g key={`${edge.from}->${edge.to}`}>{renderEdge(graph, layout, edge)}</g>)}
      {graph.nodes.map((node) => <g key={node.id}>{renderNode(graph, layout, node)}</g>)}
    </svg>
  );
}
