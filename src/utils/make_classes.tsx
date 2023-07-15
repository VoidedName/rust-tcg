export function make_classes(options: { [key: string]: boolean }): string {
    let res = "";
    for (const clazz in options) {
        if (options[clazz]) res += ` ${clazz}`
    }
    return res;
}
