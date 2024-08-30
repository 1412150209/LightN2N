import {Button, Flex, Layout, theme, Typography} from 'antd';
import {CloseOutlined, LineOutlined} from "@ant-design/icons";
import {Window} from "@tauri-apps/api/window";

const {Header} = Layout;

function header() {
    const {
        token: {borderRadiusLG, colorBgElevated, paddingXXS},
    } = theme.useToken();

    return (
        <Header data-tauri-drag-region
                style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "center",
                    padding: 10,
                    height: 40,
                    borderTopLeftRadius: borderRadiusLG,
                    borderTopRightRadius: borderRadiusLG,
                    backgroundColor: colorBgElevated
                }}>
            <Typography.Text data-tauri-drag-region style={{userSelect: "none"}}>
                Light N2N
            </Typography.Text>
            <Flex gap={paddingXXS}>
                <Button size={"small"} icon={<LineOutlined/>}
                        onClick={
                            async () => {
                                await Window.getCurrent().minimize()
                            }
                        }
                />
                <Button size={"small"} icon={<CloseOutlined/>} onClick={
                    async () => {
                        await Window.getCurrent().hide()
                    }}/>
            </Flex>
        </Header>
    )
}

export default header