import {useCallback, useEffect, useRef} from "react";

// 同步中使用异步方法
export function async_run(func: () => Promise<any>) {
    return setTimeout(async () => {
        await func()
    }, 0)
}

// 轮询
export function useInterval(callback: () => void, interval: number): () => void {
    const intervalIdRef = useRef<number | null>(null);

    const clear = useCallback(() => {
        if (intervalIdRef.current !== null) {
            clearInterval(intervalIdRef.current);
            intervalIdRef.current = null;
        }
    }, []);

    useEffect(() => {
        intervalIdRef.current = setInterval(callback, interval) as number;

        return clear;
    }, [callback, interval, clear]);

    return clear;
}

// 等待
export const Sleep = (ms: number) => {
    return new Promise(resolve => setTimeout(resolve, ms))
}

// 配置
export type Data = {
    "n2n_config": {
        "control_port": number,
        "group": string,
        "identification": string,
        "member_server": string,
        "port": number,
        "server": string
    },
    "miniserve_port": number,
    "nat_detect": Array<string>
}