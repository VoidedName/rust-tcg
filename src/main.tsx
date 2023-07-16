import './style.css'
import React, {FunctionComponent, useEffect, useState} from "react";
import {setup_main_menu_view} from "./menus/mainMenuView.tsx";
import {setup_settings_menu_view} from "./menus/settingsMenuView.tsx";
import {MapEdge, MenuAction} from "../pkg";
import {NativeObjectProps} from "./utils/props_are_same.tsx";
import {setup_pause_menu_view} from "./menus/pauseMenuView.tsx";
import {setup_map_view} from "./game/map/mapView.tsx";

const declare_foreign = <Fn extends (...args: any) => any,>(name: string) => (..._: Parameters<Fn>): ReturnType<Fn> => {
    throw new Error(`js function '${name}' has not been bound!`)
}

let render_main_menu = declare_foreign<(position: number, has_save_game: boolean) => MenuAction | undefined>("render_main_menu")

let render_settings_menu = declare_foreign<(position: number) => MenuAction | undefined>("render_settings_menu")

let render_game_map = declare_foreign<(nodes: Uint8Array[], edges: MapEdge[], current: number, visited: number[]) => void>("render_game_map")

let render_pause_menu = declare_foreign<(position: number) => MenuAction | undefined>("render_pause_menu")

let quit_application = declare_foreign<() => void>("quit_application")

let delete_save_game = () => {
    window.localStorage.clear();
}

let save_game = (state: string) => {
    window.localStorage.setItem("TCG Game", state)
}

let get_save_game = () => {
    return  window.localStorage.getItem("TCG Game") ?? undefined
}

function NullView() {
    return "Game is not Running"
}

export type ViewSetter = (set_view: (old: [FunctionComponent, NativeObjectProps]) => [FunctionComponent, NativeObjectProps]) => void
export type ViewSetup<Fn> = (setup: ViewSetter) => Fn

export function App() {
    const [[View, props], set_View] = useState<[FunctionComponent, NativeObjectProps]>(
        () => [NullView, {}]
    ) as [[FunctionComponent, NativeObjectProps], ViewSetter];

    useEffect(() => {
        render_main_menu = setup_main_menu_view(set_View);
        render_settings_menu = setup_settings_menu_view(set_View);
        render_pause_menu = setup_pause_menu_view(set_View);

        render_game_map = setup_map_view(set_View);
        quit_application = () => {
            set_View(() => [NullView, {}])
        }
    }, [])

    return <View {...props} />;
}

export {
    render_main_menu,
    render_settings_menu,
    render_game_map,
    render_pause_menu,
    quit_application,
    save_game,
    get_save_game,
    delete_save_game,
}
