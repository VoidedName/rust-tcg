import React from 'react';
import BaseGraphRenderer, { GraphLayout } from './BaseGraphRenderer';

import { Vector } from './Vector';
import { Graph, Node } from './Graph';

/* * * * * *
 * CONFIG  *
 * * * * * */
const markerHeight = 8;
const markerWidth = 4;
const nodeBaseRadius = 10;
const baseFontSize = 12;
const baseFontOffset = 4;

/* * * * * *
 * Helpers *
 * * * * * */
function computeEdge(from: Vector, to: Vector, circleOffset: number, markerOffset: number) {
  const direction = Vector.sub(to, from);
  const length = Vector.len(direction);

  if (length === 0) return null;

  const normalized = Vector.normalize(direction);

  let correctedFrom = Vector.add(from, Vector.mult(normalized, circleOffset));
  let correctedTo = Vector.sub(to, Vector.mult(normalized, markerOffset));

  const directionHasInverted = (normalized.x > 0 && correctedFrom.x > correctedTo.x)
    || (normalized.x < 0 && correctedFrom.x < correctedTo.x)
    || (normalized.y > 0 && correctedFrom.y > correctedTo.y)
    || (normalized.y < 0 && correctedFrom.y < correctedTo.y);

  if (directionHasInverted) {
    // compute distance proportional to overlap including marker offset influence
    const shrinkingLength = 1 - Vector.len(
      Vector.sub(correctedFrom, correctedTo),
    ) / (circleOffset + markerOffset);

    correctedTo = Vector.sub(to, Vector.mult(normalized, shrinkingLength * markerOffset));
    correctedFrom = from;
  }
  return [correctedFrom, correctedTo];
}

/* * * * * * *
 * Component *
 * * * * * * */
export type SimpleGraphRendererProps<TData, TGraph extends Graph<TData>> = {
  graph: TGraph,
  label: (node: Node<TData>) => string,
  layout: GraphLayout,
};

export default function SimpleGraphRenderer<TData, TGraph extends Graph<TData>>({
  graph,
  label,
  layout,
}: SimpleGraphRendererProps<TData, TGraph>) {
  const scale = 2;
  const lineWidth = scale;
  const circleRadius = nodeBaseRadius * scale;
  const fontSize = baseFontSize * scale;
  const fontOffset = baseFontOffset * scale;
  const markerOffset = (circleRadius + markerWidth * scale);
  const circleOffset = circleRadius;

  return (
      <BaseGraphRenderer
        defs={(
          <marker
            id="arrowhead"
            markerWidth={markerWidth}
            markerHeight={markerHeight}
            refY={markerHeight / 2}
            orient="auto"
          >
            <polygon points={`0 0, ${markerWidth} ${markerHeight / 2}, 0 ${markerHeight}`} />
          </marker>
      )}
        graph={graph}
        layout={layout}
        renderNode={(_, l, node: Node<TData>) => <g>
            <circle
              className="node"
              r={circleRadius}
              cx={l[node.id]!.x}
              cy={l[node.id]!.y}
            />
            <text
              className="label"
              x={l[node.id]!.x}
              y={l[node.id]!.y + fontOffset}
              fontSize={fontSize}
              textAnchor="middle"
              style={{ userSelect: 'none' }}
            >
              {label(node)}
            </text>
          </g>
        }
        renderEdge={(_, l, edge) => {
          const computedEdge = computeEdge(l[edge.from]!, l[edge.to]!, circleOffset, markerOffset);
          if (!computedEdge) return null;
          const [from, to] = computedEdge;
          return <line
              className="edge"
              x1={from!.x}
              y1={from!.y}
              x2={to!.x}
              y2={to!.y}
              strokeWidth={lineWidth}
              markerEnd="url(#arrowhead)"
            />
        }}
      />
  );
}
