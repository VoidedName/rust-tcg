export type NativeObjectProps = { [key: string]: NativeProps }
export type NativeProps = NativeObjectProps | NativeProps[] | number | string | boolean | ((..._: any[]) => any);

export function props_are_same(previous: NativeProps, now: NativeProps): boolean {
    if (previous === now) return true;
    if (previous === null || now === null) return false;
    if (Array.isArray(previous) && Array.isArray(now)) {
        if (previous.length !== now.length) return false;
        return previous.every((x, idx) => props_are_same(x, now[idx]))
    }
    if (typeof previous === "object" && typeof now === "object") {
        const previous_keys = Object.keys(previous);
        const now_keys_set = new Set(Object.keys(now));

        if (previous_keys.length !== now_keys_set.size) return false;
        return previous_keys.every(
            (x) => now_keys_set.has(x)
                && props_are_same(
                    (previous as NativeObjectProps)[x],
                    (now as NativeObjectProps)[x])
        );
    }
    return previous === now;
}
