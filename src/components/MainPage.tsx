import {Store} from "@tauri-apps/plugin-store";
import {
    Alert,
    Avatar,
    Button,
    Card,
    Col,
    Flex,
    InputNumber,
    List,
    message,
    Row,
    Space,
    Spin,
    theme,
    Typography
} from "antd";
import Global from "../config/Global.ts";
import {PoweroffOutlined} from "@ant-design/icons";
import {useEffect, useReducer, useState} from "react";
import {async_run, Data, useInterval} from "../tool/ReactTool.ts";
import multiavatar from "@multiavatar/multiavatar";
import {invoke} from "@tauri-apps/api/core";
import {Window} from "@tauri-apps/api/window";
import {resolve} from "@tauri-apps/api/path";

function MainPage() {
    const {token: {colorBgBase, colorBgElevated, colorPrimary}} = theme.useToken();

    const [data, setData] = useState<Data>()

    // n2n运行状态
    const [status, setStatus] = useState(false)

    // 异步获取外部配置
    useEffect(() => {
        async_run(async () => {
            // 存储
            const store = new Store(await resolve("client/config.json"));

            store.get<Data>("config")
                .then((d) => {
                    if (d) {
                        setData(d)
                    }
                })
        })
    }, [])

    async function SetStore(key: string, value: any) {
        // 存储
        const store = new Store(await resolve("client/config.json"));
        await store.set(key, value)
        await store.save()
    }

    // 行为
    type Action = {
        type: string,
        payload: any
    }

    // 成员列表
    function MemberList() {
        // 成员类型
        type Member = {
            info: Member_info,
            ping: number
        }
        // 成员信息
        type Member_info = {
            name: string,
            address: string,
            mode: string,
        }

        // 成员设定
        const [members, setMembers] = useReducer(refresh_members, Array<Member>())

        // ping运行状态
        const [loading, setLoading] = useState(false)

        /**
         * 刷新成员
         * @param members 原有成员信息
         * @param action
         */
        function refresh_members(members: Array<Member>, action: Action) {
            switch (action.type) {
                case "members_change":
                    let new_members = action.payload as Array<Member_info>
                    let temp = new Array<Member>()
                    for (let member of new_members) {
                        let ping = 0;
                        let o = members.find((i) => {
                            return i.info.address == member.address
                        })
                        if (o) {
                            ping = o.ping
                        }
                        temp.push({
                            info: member,
                            ping: ping
                        })
                    }
                    return temp
                case "ping":
                    let [index, ping] = action.payload as [number, number]
                    members[index].ping = ping
                    return members
                case "shutdown":
                    return []
                default:
                    return members
            }
        }

        const RandomAvatar = (s: string) => {
            return (
                <svg dangerouslySetInnerHTML={{__html: multiavatar(s)}}/>
            )
        }

        async function Ping(host: string) {
            if (host && host != "") {
                return invoke("ping_method", {host: host})
                    .catch((e) => {
                        message.error(e as string)
                    })
            }
            return 0
        }

        // 监听状态
        useEffect(() => {
            // 关闭时清空成员
            if (!status) {
                setMembers({
                    type: "shutdown",
                    payload: []
                })
            }
        }, [status])

        // 轮询 成员变化
        useInterval(() => {
            // 未运行不执行
            if (status) {
                // 仅窗口在前台时刷新成员
                Window.getCurrent().isVisible()
                    .then(async (flag) => {
                        if (flag) {
                            await invoke("n2n_members")
                                .then((m) => {
                                    if (typeof m == "object") {
                                        let mem = (m as Array<Member_info>)
                                        // 刷新成员
                                        setMembers({
                                            type: "members_change",
                                            payload: mem
                                        })
                                    }
                                })
                                .catch((e) => {
                                    message.error(e as string)
                                })
                        }
                    })
            }
        }, 5000)

        return (
            <div
                className={"scroll-content"}
                style={{
                    overflow: "auto",
                    height: 190,
                }}>
                <List
                    style={{
                        userSelect: "none",
                        background: colorBgElevated,
                    }}
                    locale={{emptyText: "没有成员"}}
                    itemLayout={"horizontal"}
                    dataSource={members}
                    renderItem={(item, index) => (
                        <List.Item key={item.ping}>
                            <List.Item.Meta
                                style={{userSelect: "none", alignItems: "center"}}
                                avatar={<Avatar
                                    src={RandomAvatar(item.info.name + index)}
                                    alt={index.toString()}
                                    draggable={"false"}>
                                </Avatar>
                                }
                                title={
                                    <Typography.Text>
                                        {item.info.name}
                                        <Typography.Text style={{marginLeft: 10}}
                                                         type={item.ping > 0 ? (item.ping > 100 ? "warning" : "success") : ("danger")}>
                                            {item.ping != 0 ? item.ping + "ms" : null}
                                        </Typography.Text>
                                    </Typography.Text>
                                }
                                description={
                                    <Typography style={{marginTop: -4}}>
                                        <Typography.Text
                                            style={{marginRight: 5}}
                                            type={item.info.mode != "p2p" ? "danger" : "success"}>
                                            {item.info.mode}
                                        </Typography.Text>
                                        <Typography.Text
                                            copyable={true}>
                                            {item.info.address.split("/")[0]}
                                        </Typography.Text>
                                        <Typography.Link
                                            style={{marginLeft: 15}}
                                            disabled={item.info.address == "None" || loading}
                                            onClick={() => {
                                                setLoading(true)
                                                Ping(item.info.address.split("/")[0])
                                                    .then((s) => {
                                                        if (typeof s == "number" && members[index].ping != s) {
                                                            setMembers({
                                                                type: "ping",
                                                                payload: [index, s]
                                                            })
                                                        }
                                                    })
                                                    .finally(() => {
                                                        setLoading(false)
                                                    })
                                            }}
                                        >
                                            Ping
                                        </Typography.Link>
                                        <Spin size={"small"} spinning={loading}/>
                                    </Typography>
                                }
                            />
                        </List.Item>
                    )}
                />
            </div>
        )
    }

    // 组选择
    function GroupSelect() {
        const [group_num, setGroup_num] = useReducer(group_change, 10)

        // 异步获取外部配置
        useEffect(() => {
            if (data) {
                setGroup_num({
                    type: "init",
                    payload: data.n2n_config.group.slice(4) as unknown as number
                })
            }
        }, [])

        function group_change(num: number, action: Action) {
            switch (action.type) {
                case "change":
                    if (typeof action.payload == "number") {
                        async_run(async () => {
                            if (data) {
                                data.n2n_config.group = "lers" + action.payload
                                await SetStore("config", data)
                            }
                        })
                        return action.payload
                    }
                    break
                case "init":
                    if (typeof action.payload == "number") {
                        return action.payload
                    }
            }
            return num
        }

        return (
            <Row align={"middle"} style={{marginLeft: 10, marginRight: 10, userSelect: "none"}}>
                <Col span={3}>
                    <Typography.Text>
                        组:
                    </Typography.Text>
                </Col>
                <Col span={21}>
                    <InputNumber
                        disabled={status}
                        size={"small"}
                        addonBefore={"lers"}
                        min={1}
                        max={99}
                        value={group_num}
                        onChange={(t) => {
                            if (t != null) {
                                setGroup_num({
                                    type: "change",
                                    payload: t
                                })
                            }
                        }}
                    />
                </Col>
            </Row>
        )
    }

    // 按钮
    function ButtonTop() {
        // 按钮加载状态
        const [loading, setLoading] = useState(false)
        // 自己的虚拟ip
        const [ip, setIP] = useState("0.0.0.0")
        // 自己的标识
        const [name, setName] = useState("Default")

        /**
         * 按钮标识
         */
        function ButtonIcon() {
            let text = "OFF"
            let color = "#E1341E"
            if (status) {
                text = "ON"
                color = "#1ECBE1"
            }
            return (
                <>
                    <PoweroffOutlined style={{
                        fontSize: 28,
                        color: color
                    }}/>
                    <Typography.Title level={5} style={{margin: 0}}>{text}</Typography.Title>
                </>
            )
        }

        // 异步获取外部配置
        useEffect(() => {
            if (data) {
                setName(data.n2n_config.identification)
            }
        }, [])

        // 监测状态
        useEffect(() => {
            if (status) {
                // 更新ip
                async_run(async () => {
                    try {
                        const ip = await invoke("n2n_self_ip")
                        if (typeof ip == "string") {
                            setIP(ip)
                        }
                    } catch (e) {
                        message.error(e as string)
                    }
                })
            } else {
                setIP("0.0.0.0")
            }
        }, [status])


        /**
         * 启动n2n客户端
         */
        async function start_client() {
            if (await check_adapter()) {
                await invoke("n2n_client_start")
                    .then(async (f) => {
                        if (f as boolean) {
                            await invoke("n2n_status")
                                .then((s) => {
                                    setStatus(s as boolean);
                                })
                                .catch((e) => {
                                    message.error(e as string)
                                })
                        }
                    })
                    .catch((e) => {
                        message.error(e as string)
                    })
            }
        }

        /**
         * 停止n2n客户端
         */
        async function stop_client() {
            await invoke("n2n_client_stop")
                .then(async (f) => {
                    if (f as boolean) {
                        await invoke("n2n_status")
                            .then((s) => {
                                setStatus(s as boolean);
                            })
                    }
                })
        }

        /**
         * 检查虚拟网卡安装状态
         */
        async function check_adapter() {
            try {
                let result = await invoke("n2n_check_adapter");
                if (typeof result == "boolean") {
                    if (!result) {
                        message.error("请手动安装此虚拟网卡")
                        open("https://build.openvpn.net/downloads/releases/tap-windows-9.24.7-I601-Win10.exe", "_blank")
                    }
                    return result
                }
            } catch (e) {
                console.error(e)
                message.error(e as string)
            }
            return false
        }


        /**
         * 检查防火墙情况,并自动添加规则
         */
        async function check_firewall() {
            let status = false
            await invoke("n2n_firewall_check")
                .then(async (s) => {
                    // 如果不存在防火墙
                    if (!s) {
                        // 添加防火墙
                        await invoke("n2n_firewall_add")
                            .catch((e) => {
                                message.error(e as string)
                            })
                        await invoke("n2n_firewall_check")
                            .then((s) => {
                                status = s as boolean
                            })
                            .catch((e) => {
                                message.error(e as string)
                            })
                    } else {
                        status = s as boolean
                    }
                })
                .catch((e) => {
                    message.error(e as string)
                })
            return status
        }

        return (
            <Card style={{
                width: Global.ThemeCss.cardWidth,
                backgroundColor: colorBgBase
            }}>
                <Flex align={"center"}>
                    <Button
                        style={{height: 80, width: 80}}
                        shape={"round"}
                        icon={<ButtonIcon/>}
                        disabled={loading}
                        loading={loading}
                        onClick={(event) => {
                            event.stopPropagation()
                            if (!status) {
                                async_run(async () => {
                                    setLoading(true)
                                    // 检查防火墙
                                    let s = await check_firewall()
                                    if (s) {
                                        // 保存配置
                                        if (data) {
                                            data.n2n_config.identification = name
                                            await SetStore("config", data)
                                        }
                                        await start_client()
                                    }
                                    setLoading(false)
                                })
                            } else {
                                // 关闭n2n
                                async_run(async () => {
                                    setLoading(true)
                                    await stop_client()
                                        .finally(() => {
                                            setTimeout(() => {
                                                setLoading(false)
                                            }, 1000)
                                        })
                                })
                            }
                        }}
                    />
                    <Typography style={{marginLeft: 20}}>
                        <Typography.Paragraph
                            style={{
                                marginBottom: 0,
                                userSelect: "none",
                                fontSize: 18
                            }}
                            type={"secondary"}
                            editable={status ? false : {
                                maxLength: 8,
                                onChange: (s) => {
                                    setName(s)
                                },
                                enterIcon: false,
                            }}
                        >
                            {name}
                        </Typography.Paragraph>
                        <Typography.Paragraph
                            style={{
                                userSelect: "none",
                                marginBottom: 0,
                                color: colorPrimary
                            }}
                            copyable={!status ? false : {
                                format: "text/plain",
                                text: ip
                            }}
                        >
                            {ip}
                        </Typography.Paragraph>
                    </Typography>
                </Flex>
            </Card>
        )
    }

    // ICMP回显提示
    function ICMPAlert() {
        const [isShow, setShow] = useState(false)
        const [loading, setLoading] = useState(false)

        // 监听状态
        useEffect(() => {
            if (status) {
                async_run(async () => {
                    await CheckFireWallRule()
                })
            }
        }, [status])

        async function CheckFireWallRule() {
            invoke("ping_firewall_rule_check")
                .then((s) => {
                    setShow(!s)
                })
                .catch((e) => {
                    message.error(e as string)
                })
        }

        // async function RemoveFireWallRule() {
        //     if (isShow) {
        //         invoke("ping_firewall_rule_rm")
        //             .then(() => {
        //                 CheckFireWallRule()
        //             })
        //             .catch((e) => {
        //                 message.error(e as string)
        //             })
        //     }
        // }

        async function AddFireWallRule() {
            if (isShow) {
                invoke("ping_firewall_rule_add")
                    .then(() => {
                        CheckFireWallRule()
                    })
                    .catch((e) => {
                        message.error(e as string)
                    })
            }
        }

        return (
            <>
                {isShow && <Alert
                    message={"未开启ICMP回显,无法被ping通"}
                    type={"error"}
                    action={
                        <Button loading={loading} onClick={() => {
                            setLoading(true)
                            AddFireWallRule()
                                .finally(() => {
                                    setLoading(false)
                                })
                        }}>
                            开启
                        </Button>
                    }
                />}
            </>
        )
    }

    return (
        <Space direction={"vertical"} style={{paddingLeft: 10, paddingRight: 10, paddingTop: 5}}>
            <ButtonTop/>
            <GroupSelect/>
            <ICMPAlert/>
            <MemberList/>
        </Space>
    )
}

export default MainPage