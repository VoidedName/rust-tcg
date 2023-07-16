import {GameMapAction, MapNode} from "../../../pkg"
import React, {FunctionComponent, useEffect} from "react";
import {render_game_map, ViewSetup} from "../../main.tsx";
import {props_are_same} from "../../utils/props_are_same.tsx";
import "./mapView.scss"
import SimpleGraphRenderer from "../../math/SimpleGraphRenderer.tsx";
import {GraphLayout} from "../../math/BaseGraphRenderer.tsx";

let map_action: GameMapAction = "Waiting";

function set_map_action(action: GameMapAction) {
    map_action = action;
}

type GameMapProps = {
    nodes: MapNode[],
    edges: [number, number][],
    position: number,
    visited: number[],
    consume_action: (action: GameMapAction) => void
}
const KeyToMainMenuAction = {
    "Escape": "PauseGame",
} as const;

function generate_layout(edges: [number, number][]): GraphLayout {
    const edge_map: Record<number, Set<number>> = {};
    for (let [from, to] of edges) {
        if (edge_map[from] === undefined) {
            edge_map[from] = new Set();
        }
        edge_map[from].add(to);
    }

    let max_in_layer = 0;
    const layers = [[0]];

    for (let layer of layers) {
        let next_layer_seen = new Set();
        let next_layer = [];
        for (let node of layer) {
            if (node in edge_map) {
                for (let to of edge_map[node]) {
                    if (!next_layer_seen.has(to)) {
                        next_layer_seen.add(to)
                        next_layer.push(to)
                    }
                }
            }
        }
        if (next_layer.length > 0) {
            layers.push(next_layer)
        }
        max_in_layer = Math.max(max_in_layer, next_layer.length)
    }

    const space = max_in_layer * 100;
    return Object.fromEntries(layers.flatMap(
        (layer, x) => {
            const distance_between_nodes = space / layer.length;
            const offset = distance_between_nodes / 2;
            return layer.map(
                (node, y) => {
                    return ([node, {x: x * 100 + 100, y: y * distance_between_nodes + offset + 100}])
                }
            )
        }
    ))
}

export function MapView({consume_action, edges, nodes, visited, position}: GameMapProps) {
    useEffect(() => {
        const l = (e: KeyboardEvent) => {
            if (e.key in KeyToMainMenuAction && !e.repeat) {
                consume_action(KeyToMainMenuAction[e.key as keyof typeof KeyToMainMenuAction])
            }
        };
        window.addEventListener("keydown", l);
        return () => window.removeEventListener("keydown", l);
    })

    let layout = generate_layout(edges);

    return <div className={"map"}><SimpleGraphRenderer
        graph={{
            nodes: nodes.map((x, id) => ({id, data: x})),
            edges: edges.map(x => ({from: x[0], to: x[1]})),
        }}
        layout={layout}
        label={(n) => "" + n.id}
    /></div>
}

export const setup_map_view: ViewSetup<typeof render_game_map> = (setView) => (nodes, edges, position, visited) => {
    setView(view => {
        let other_view = view[0] !== MapView;
        let props_changed = !props_are_same(view[1], {
            nodes: [...nodes] as unknown as MapNode[],
            edges,
            position,
            visited,
            consume_action: set_map_action
        });
        if (other_view || props_changed) {
            map_action = "Waiting";
            return [MapView as FunctionComponent, {
                nodes: [...nodes] as unknown as MapNode[],
                edges,
                position,
                visited,
                consume_action: set_map_action
            }]
        }
        return view;
    });
    const tmp = map_action;
    map_action = "Waiting";
    return tmp;
}
