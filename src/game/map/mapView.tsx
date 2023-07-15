import {GameMapActions} from "../../../pkg";
import React, {FunctionComponent, useEffect} from "react";
import {ViewSetup} from "../../main.tsx";
import {props_are_same} from "../../utils/props_are_same.tsx";
import {render_game_map} from "../../main.tsx";

let map_action: GameMapActions | undefined = undefined;

function set_map_action(action: GameMapActions | undefined) {
    map_action = action;
}

type GameMapProps = { position: number, consume_action: (action: GameMapActions) => void }
const KeyToMainMenuAction = {
    "Escape": GameMapActions.PauseGame,
} as const;

export function MapView({consume_action}: GameMapProps) {
    useEffect(() => {
        const l = (e: KeyboardEvent) => {
            if (e.key in KeyToMainMenuAction && !e.repeat) {
                consume_action(KeyToMainMenuAction[e.key as keyof typeof KeyToMainMenuAction])
            }
        };
        window.addEventListener("keydown", l);
        return () => window.removeEventListener("keydown", l);
    })

    return "Game Map Goes Here, ESC to Pause"
}

export const setup_map_view: ViewSetup<typeof render_game_map> = (setView) => () => {
    setView(view => {
        let other_view = view[0] !== MapView;
        let props_changed = !props_are_same(view[1], {
            consume_action: set_map_action
        });
        if (other_view || props_changed) {
            map_action = undefined;
            return [MapView as FunctionComponent, {
                consume_action: set_map_action
            }]
        }
        return view;
    });
    const tmp = map_action;
    map_action = undefined;
    return tmp;
}
