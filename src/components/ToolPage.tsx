import React, {JSX, useEffect, useState} from "react";
import {Button, Card, Col, Flex, List, message, Row, Select, Space, Typography} from "antd";
import Global from "../config/Global.ts";
import {BaseType} from "antd/es/typography/Base";
import {CheckOutlined, QuestionCircleOutlined, StopOutlined} from "@ant-design/icons";
import {invoke} from "@tauri-apps/api/core";
import {open} from '@tauri-apps/plugin-shell';
import {open as opend} from '@tauri-apps/plugin-dialog';
import {async_run, Data} from "../tool/ReactTool.ts";
import {Store} from "@tauri-apps/plugin-store";
import {resolve} from "@tauri-apps/api/path";

function ToolPage() {

    function NatDetector() {
        const [nat_type, setNatType] = React.useState("Unknown")
        const [loading, setLoading] = React.useState(false)
        const [other_type, setOtherType] = React.useState("Unknown")
        const [dig, setDig] = React.useState(-1)

        const color = new Map([
            ["OpenInternet", "success"],
            ["FullCone", "success"],
            ["RestrictedCone", "warning"],
            ["PortRestrictedCone", "warning"],
            ["Symmetric", "danger"],
            ["Unknown", "secondary"]
        ])

        const value = new Map([
            ["OpenInternet", 3],
            ["FullCone", 2],
            ["RestrictedCone", 1],
            ["PortRestrictedCone", 0.5],
            ["Symmetric", -1],
            ["Unknown", -2]
        ])

        function IfDig(one: string, another: string) {
            setDig((value.get(one) as number) + (value.get(another) as number))
        }

        function GetStatus() {
            if (dig != 0) {
                return (
                    <Typography.Text type={dig > 0 ? "success" : "danger"}>
                        {dig > 0 ? "能联通" : "联不通"}
                        {dig > 0 ? <CheckOutlined/> : <StopOutlined/>}
                    </Typography.Text>
                )
            } else {
                return (
                    <Typography.Text type={"warning"}>
                        有可能联通<QuestionCircleOutlined/>
                    </Typography.Text>
                )
            }
        }

        async function NatDetect() {
            try {
                setLoading(true)
                await invoke("nat_detect")
                    .then((t) => {
                        setNatType(t as string)
                    })
            } catch (e) {
                message.error({
                    content: e as string
                })
            } finally {
                setLoading(false)
            }
        }

        return (
            <Card
                style={{width: Global.ThemeCss.cardWidth, userSelect: "none"}}
                styles={{
                    body: {
                        paddingTop: 15,
                        paddingBottom: 5
                    }
                }}
                title={<p style={{fontSize: 14}}>NAT类型判断工具</p>}
                actions={[<Button onClick={async () => {
                    await NatDetect()
                }} loading={loading} disabled={loading}>开始检测</Button>]}
            >
                <Row align={"middle"}>
                    <Col span={7}>
                        <Typography.Text type={"secondary"}>
                            NAT类型:
                        </Typography.Text>
                    </Col>
                    <Col span={15} offset={2}>
                        <Typography.Text type={color.get(nat_type) as BaseType}>
                            {nat_type}
                        </Typography.Text>
                    </Col>
                </Row>
                <Row style={{marginTop: 5}}>
                    <Col span={7}>
                        <Typography.Text type={"secondary"}>
                            对方类型:
                        </Typography.Text>
                    </Col>
                    <Col span={15} offset={2}>
                        <Select
                            style={{marginLeft: -8, width: 155}}
                            size={"small"}
                            options={Array.from(color.keys()).map((key) => ({
                                label: key,
                                value: key
                            }))}
                            onChange={(t) => {
                                setOtherType(t)
                                IfDig(t, nat_type)
                            }}
                            value={other_type}
                        />
                    </Col>
                    <Col style={{marginTop: 5}} span={24}>
                        <Flex justify={"center"}>
                            <GetStatus/>
                        </Flex>
                    </Col>
                </Row>
            </Card>
        )
    }

    function WinIPBroadcast() {
        const [status, setStatus] = React.useState(false);
        const [loading, setLoading] = React.useState(false)

        /**
         * 启动程序
         * @constructor
         */
        async function StartBroadcast() {
            try {
                setLoading(true);
                await invoke("win_ip_broadcast_start")
                    .then(async (res) => {
                        if (res as boolean) {
                            await CheckStatus()
                        }
                    })
            } catch (e) {
                message.error({
                    content: e as string
                })
            } finally {
                setLoading(false)
            }

        }

        /**
         * 更新运行状态
         * @constructor
         */
        async function CheckStatus() {
            await invoke("win_ip_broadcast_status")
                .then((output) => {
                    if (typeof output == "boolean") {
                        setStatus(output);
                    } else {
                        setStatus(false)
                    }
                })
        }

        /**
         * 关闭程序
         * @constructor
         */
        async function StopBroadcast() {
            try {
                setLoading(true);
                await invoke("win_ip_broadcast_stop")
                    .then(() => {
                        CheckStatus()
                    });
            } catch (e) {
                message.error({
                    content: e as string
                })
            } finally {
                setLoading(false)
            }
        }

        return (
            <Card
                style={{width: Global.ThemeCss.cardWidth, userSelect: "none"}}
                styles={{
                    body: {
                        paddingTop: 15,
                        paddingBottom: 5
                    }
                }}
                title={<>
                    <p style={{fontSize: 14}}>WinIPBroadcast工具</p>
                </>}
                actions={[<Button onClick={async () => {
                    if (status) {
                        await StopBroadcast();
                    } else {
                        await StartBroadcast();
                    }
                }} loading={loading} disabled={loading}>{status ? "关闭" : "启动"}</Button>]}
            >
                <Row align={"middle"}>
                    <Col>
                        <Typography.Text type={"secondary"}>
                            说明:
                        </Typography.Text>
                    </Col>
                    <Col>
                        <Typography.Text>
                            WinIPBroadcast，它会在本地监听并获取所有的全局广播数据包，然后重发到每一个网络接口上。这样就能让数据包广播到虚拟局域网，虚拟局域网内的其它客户端就能接收到房间信息，这样就能搜到房间了。
                        </Typography.Text>
                    </Col>
                    <Col span={24}>
                        <Flex justify={"center"}>
                            {status ?
                                <Typography.Text type={"success"}>
                                    Running
                                    <CheckOutlined/>
                                </Typography.Text> :
                                <Typography.Text type={"danger"}>
                                    Stopping
                                    <StopOutlined/>
                                </Typography.Text>
                            }
                        </Flex>
                    </Col>
                </Row>
            </Card>
        )
    }

    function LocalShare() {
        return (
            <Card
                style={{width: Global.ThemeCss.cardWidth, userSelect: "none"}}
                styles={{
                    body: {
                        paddingTop: 15,
                        paddingBottom: 5
                    }
                }}
                title={<>
                    <p style={{fontSize: 14}}>局域网文件分享工具</p>
                </>}
                actions={[
                    <Button onClick={async () => {
                        await open("http://share.lers.fun")
                    }}>打开页面</Button>
                ]}
            >
                <Row align={"middle"}>
                    <Col>
                        <Typography.Text type={"secondary"}>
                            说明:
                        </Typography.Text>
                    </Col>
                    <Col>
                        <Typography.Text>
                            Snapdrop是一款可以实现跨平台传递文件的工具，用户只要有浏览器就能进行使用。SnapDrop只需要同时打开一个网页,就能传输文件了,不会在任何服务器端保存数据,
                            P2P 传输, 基于浏览器的 WebRTC 接口, 手机端与电脑端均可使用.
                        </Typography.Text>
                    </Col>
                </Row>
            </Card>
        )
    }

    function MiniServe() {
        const [isSharing, setSharing] = useState(false)
        const [port, setPort] = useState(8090)

        // 异步获取外部配置
        useEffect(() => {
            async_run(async () => {
                // 存储
                const store = new Store(await resolve("client/config.json"));

                store.get<Data>("config")
                    .then((d) => {
                        if (d) {
                            setPort(d.miniserve_port)
                        }
                    })
            })
        }, [])

        async function CheckFirewall() {
            let status = false
            await invoke("miniserve_firewall_check")
                .then(async (s) => {
                    if (!s) {
                        await invoke("miniserve_firewall_add")
                            .catch((e) => {
                                message.error(e as string)
                            })
                        await invoke("miniserve_firewall_check")
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

        async function StartShare(path: string) {
            // 不能同时多个分享
            if (!isSharing) {
                // 检查防火墙
                setSharing(true)
                if (await CheckFirewall()) {
                    await invoke("miniserve_start", {path: path})
                        .catch((e) => {
                            message.error(e as string)
                            setSharing(false)
                        })
                }
            }
        }

        async function StopShare() {
            if (isSharing) {
                await invoke("miniserve_stop")
                    .then(() => {
                        setSharing(false)
                    })
                    .catch((e) => {
                        message.error(e as string)
                    })
            }
        }

        const Buttons = () => {
            if (!isSharing) {
                return [
                    <Button disabled={isSharing} onClick={async () => {
                        let file = await opend({
                            title: "选择一个文件共享",
                            directory: false,
                            multiple: false
                        })
                        if (file) {
                            await StartShare(file.path)
                        }
                    }}>选择单文件</Button>,
                    <Button disabled={isSharing} onClick={async () => {
                        let path = await opend({
                            title: "选择一个文件夹共享",
                            directory: true,
                            multiple: false
                        })
                        if (path) {
                            await StartShare(path)
                        }
                    }}>选择文件夹</Button>

                ]
            } else {
                return [<Button disabled={!isSharing} onClick={async () => {
                    await StopShare()
                }}>停止共享</Button>]
            }
        }

        return (
            <Card
                style={{width: Global.ThemeCss.cardWidth, userSelect: "none"}}
                styles={{
                    body: {
                        paddingTop: 15,
                        paddingBottom: 5
                    }
                }}
                title={<>
                    <p style={{fontSize: 14}}>MiniServe</p>
                </>}
                actions={Buttons()}
            >
                <Row align={"middle"}>
                    <Col>
                        <Typography.Text type={"secondary"}>
                            说明:
                        </Typography.Text>
                    </Col>
                    <Col>
                        <Typography.Text>
                            miniserve 是一个小型、独立的跨平台 CLI 工具，允许您通过网页共享文件。
                        </Typography.Text>
                    </Col>
                </Row>
                <Row align={"middle"} justify={"center"}>
                    {isSharing && <Typography.Text type={"secondary"}>
                        正在端口<Typography.Text type={"success"}>{port}</Typography.Text>上共享
                    </Typography.Text>}
                </Row>
                <Row>
                    <Col span={24}>
                        <Flex justify={"center"}>
                            {isSharing ?
                                <Typography.Text type={"success"}>
                                    Running
                                    <CheckOutlined/>
                                </Typography.Text> :
                                <Typography.Text type={"danger"}>
                                    Stopping
                                    <StopOutlined/>
                                </Typography.Text>
                            }
                        </Flex>
                    </Col>
                </Row>
            </Card>
        )
    }

    type Tool = {
        name: string,
        element: JSX.Element
    }

    const Tools: Tool[] = [
        {
            name: "NatDetect",
            element: <NatDetector/>
        },
        {
            name: "MiniServe",
            element: <MiniServe/>
        },
        {
            name: "WinIPBroadcast",
            element: <WinIPBroadcast/>
        },
        {
            name: "LocalShare",
            element: <LocalShare/>
        }
    ]

    return (
        <Space direction={"vertical"} style={{paddingLeft: 10, paddingRight: 10, height: 365, overflow: "auto"}}>
            <List
                locale={{
                    emptyText: "没有工具"
                }}
                dataSource={Tools}
                renderItem={(item) => (
                    <List.Item>
                        {item.element}
                    </List.Item>
                )}
            />
        </Space>
    )
}

export default ToolPage