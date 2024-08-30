import {App, ConfigProvider, Layout, theme as themes} from 'antd';
import Header from "./components/Header.tsx";
import {useContext, useLayoutEffect} from "react";
import Content from "./components/Content.tsx";
import {StyleProvider} from '@ant-design/cssinjs';

import zhCN from 'antd/locale/zh_CN';
import UpdateModal from "./components/UpdateModal.tsx";

function Main() {

    const {locale, theme} = useContext(ConfigProvider.ConfigContext);

    useLayoutEffect(() => {
        ConfigProvider.config({
            holderRender: (children) => (
                <StyleProvider hashPriority="high">
                    <ConfigProvider prefixCls="static" iconPrefixCls="icon" locale={locale} theme={theme}>
                        <App notification={{maxCount: 3}} message={{maxCount: 5}}>
                            {children}
                        </App>
                    </ConfigProvider>
                </StyleProvider>
            ),
        });
    }, [locale, theme]);

    return (
        <ConfigProvider
            locale={zhCN}
            theme={{
                cssVar: true,
                algorithm: themes.darkAlgorithm
            }}
        >
            <Layout onContextMenu={(e) => {
                e.preventDefault()
            }} style={{background: "transparent", overflow: "hidden"}}>
                <UpdateModal/>
                <Header/>
                <Content/>
            </Layout>
        </ConfigProvider>
    );
}

export default Main;