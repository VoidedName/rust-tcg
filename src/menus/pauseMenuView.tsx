import {MenuAction, PauseMenu} from "../../pkg";
import React, {FunctionComponent, useEffect} from "react";
import "./menu.scss"
import {render_settings_menu, ViewSetup} from "../main.tsx";
import {make_classes} from "../utils/make_classes.tsx";
import {props_are_same} from "../utils/props_are_same.tsx";

let menu_action: MenuAction | undefined = undefined;

function set_main_menu_action(action: MenuAction | undefined) {
    menu_action = action;
}

type SettingsMenuViewProps = { position: number, consume_action: (action: MenuAction) => void }
const KeyToMainMenuAction = {
    "ArrowUp": MenuAction.Previous,
    "ArrowDown": MenuAction.Next,
    "Enter": MenuAction.Confirm,
} as const;

const settings_menu = Object.values(PauseMenu).filter(x => !isNaN(Number(x))).sort() as number[]

export function PauseMenuView({position, consume_action}: SettingsMenuViewProps) {
    useEffect(() => {
        const l = (e: KeyboardEvent) => {
            if (e.key in KeyToMainMenuAction && !e.repeat) {
                consume_action(KeyToMainMenuAction[e.key as keyof typeof KeyToMainMenuAction])
            }
        };
        window.addEventListener("keydown", l);
        return () => window.removeEventListener("keydown", l);
    })

    return <ul className={"menu"}>
        {settings_menu.map((item) => <li key={item}>
            <button
                className={make_classes({"selected": position === item})}
            >
                {PauseMenu[item]}
            </button>
        </li>)}
    </ul>
}

export const setup_pause_menu_view: ViewSetup<typeof render_settings_menu> = (setView) => (position) => {
    setView(view => {
        let other_view = view[0] !== PauseMenuView;
        let props_changed = !props_are_same(view[1], {
            position,
            consume_action: set_main_menu_action
        });
        if (other_view || props_changed) {
            menu_action = undefined;
            return [PauseMenuView as FunctionComponent, {
                position,
                consume_action: set_main_menu_action
            }]
        }
        return view;
    });
    const tmp = menu_action;
    menu_action = undefined;
    return tmp;
}
