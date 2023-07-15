import ReactDOM from "react-dom/client";
import React from "react";
import {App} from "./main.tsx";
import init from "../pkg";

init().then(() => {
    ReactDOM.createRoot(document.getElementById('root')!).render(
        <App/>
    )
});
