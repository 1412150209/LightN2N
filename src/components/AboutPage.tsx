import {Card, Divider, Flex, theme, Typography} from "antd";

function AboutPage() {
    const {token: {colorBgElevated},} = theme.useToken();
    return (
        <Flex vertical={true} style={{height: 368, overflow: "auto"}}>
            <Card bordered={false} styles={{
                body: {
                    paddingTop: 20,
                    paddingBottom: 0,
                    paddingLeft: 15,
                    background: colorBgElevated
                }
            }}>
                <Typography>
                    <Typography.Title level={4} style={{marginTop: 0, marginBottom: 5}}>
                        软件简介
                    </Typography.Title>
                    <Typography.Paragraph>
                        <Typography.Text>
                            Little N2N是一款用于组建虚拟局域网的软件，核心使用了
                            <Typography.Link href={"https://github.com/ntop/n2n"} target={"_blank"}>
                                n2n
                            </Typography.Link>
                            这个开源项目，旨在用最简单的方式创建最方便的联机环境。
                        </Typography.Text>
                    </Typography.Paragraph>
                    <Divider style={{marginTop: 8, marginBottom: 8}}/>
                    <Typography.Title level={4} style={{marginTop: 0, marginBottom: 5}}>
                        软件作者
                    </Typography.Title>
                    <Typography.Link href={"https://lers.fun/"} target={"_blank"}>
                        @lers梦魔
                    </Typography.Link>
                    <Divider style={{marginTop: 8, marginBottom: 8}}/>
                    <Typography.Title level={4} style={{marginTop: 0, marginBottom: 5}}>
                        特别感谢
                    </Typography.Title>
                    <Flex vertical={true}>
                        <Typography.Link href={"https://github.com/ntop/n2n"} target={"_blank"}>
                            n2n@github
                        </Typography.Link>
                        <Typography.Link href={"https://github.com/lucktu/n2n"} target={"_blank"}>
                            lucktu/n2n@github
                        </Typography.Link>
                        <Typography.Link href={"https://ant-design.antgroup.com/"} target={"_blank"}>
                            Ant Design
                        </Typography.Link>
                        <Typography.Link href={"https://tauri.app/"} target={"_blank"}>
                            Tauri
                        </Typography.Link>
                        <Typography.Link href={"https://github.com/dechamps/WinIPBroadcast"} target={"_blank"}>
                            WinIPBroadcast
                        </Typography.Link>
                    </Flex>
                </Typography>
            </Card>
        </Flex>
    )
}

export default AboutPage