import {Layout, Tabs, TabsProps, theme} from "antd";
import MainPage from "./MainPage.tsx";
import ToolPage from "./ToolPage.tsx";
import AboutPage from "./AboutPage.tsx";

const {Content} = Layout;

function content() {
    const {token: {colorBgElevated, borderRadiusLG},} = theme.useToken();

    const items: TabsProps['items'] = [
        {
            key: '1',
            label: <p style={{userSelect: "none", marginTop: 0, marginBottom: 0}}>组网</p>,
            children: <MainPage key={1}/>,
        },
        {
            key: '2',
            label: <p style={{userSelect: "none", marginTop: 0, marginBottom: 0}}>工具</p>,
            children: <ToolPage key={2}/>
        },
        {
            key: '3',
            label: <p style={{userSelect: "none", marginTop: 0, marginBottom: 0}}>关于</p>,
            children: <AboutPage key={3}/>,
        },
    ];

    return (
        <Content
            style={{
                background: colorBgElevated,
                height: 420,
                borderBottomLeftRadius: borderRadiusLG,
                borderBottomRightRadius: borderRadiusLG,
            }}>
            <Tabs centered
                  defaultActiveKey="1"
                  items={items}
                  tabBarStyle={{
                      marginBottom: 0
                  }}
                  style={{
                      background: colorBgElevated
                  }}
                  tabBarGutter={60}
            />
        </Content>
    )
}

export default content